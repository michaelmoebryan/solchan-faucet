import { describe, it, before } from "mocha";
import { assert } from "chai";
import * as anchor from "@project-serum/anchor";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
import { SolchanFaucet } from "../target/types/solchan_faucet";  // Adjust the import path as necessary

describe("solchan-faucet", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolchanFaucet as anchor.Program<SolchanFaucet>;

  const faucetAccount = Keypair.generate();
  console.log("Generated faucet account:", faucetAccount.publicKey.toBase58());

  const userAccount = Keypair.generate();
  console.log("Generated user account:", userAccount.publicKey.toBase58());

  const admin = Keypair.generate();
  console.log("Admin public key:", admin.publicKey.toBase58());

  before(async () => {
    console.log("Setting up the environment...");

    const airdropSignature = await provider.connection.requestAirdrop(admin.publicKey, 1000000000);
    await provider.connection.confirmTransaction(airdropSignature, "confirmed");
    console.log("Airdropped 10 SOL to admin account");

    const airdropFaucetSignature = await provider.connection.requestAirdrop(faucetAccount.publicKey, 20 * anchor.web3.LAMPORTS_PER_SOL); // 20 SOL
    await provider.connection.confirmTransaction(airdropFaucetSignature, "confirmed");
    console.log("Airdropped 20 SOL to faucet account");

    try {
      await program.methods.initializeFaucet()
        .accounts({
          faucet: faucetAccount.publicKey,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([faucetAccount, admin])
        .rpc();
      console.log("Initialized the faucet successfully");
    } catch (err) {
      console.error("Error during initializeFaucet:", err);
      throw err;
    }
  });

  it("Requests funds from the faucet", async () => {
    console.log("Requesting funds from the faucet...");

    try {
      await program.methods.requestFunds()
        .accounts({
          faucet: faucetAccount.publicKey,
          user: userAccount.publicKey,
          admin: admin.publicKey, // Added admin account as required by the updated contract
          systemProgram: SystemProgram.programId,
        })
        .signers([userAccount]) // The user is the one requesting funds, so userAccount should be the signer
        .rpc();
      console.log("Requested funds successfully");
    } catch (err) {
      console.error("Error during requestFunds:", err);
      throw err;
    }

    const userAccountData = await provider.connection.getAccountInfo(userAccount.publicKey);
    console.log("User account data:", userAccountData);

    assert(userAccountData !== null);

    const faucetData = await program.account.faucet.fetch(faucetAccount.publicKey);
    console.log("Updated faucet account data:", faucetData);

    assert.isAbove(faucetData.lastRequestTime.toNumber(), 0);
  });

  it("Fails when requesting funds too soon", async () => {
    console.log("Testing failure case for requesting funds too soon...");

    try {
      await program.methods.requestFunds()
        .accounts({
          faucet: faucetAccount.publicKey,
          user: userAccount.publicKey,
          admin: admin.publicKey, // Added admin account as required by the updated contract
          systemProgram: SystemProgram.programId,
        })
        .signers([userAccount])
        .rpc();
      assert.fail("Request should have failed due to requesting too soon");
    } catch (err) {
      console.error("Expected error during requestFunds (too soon):", err.message);
      if (err.error && err.error.errorCode) {
        assert.equal(err.error.errorCode.code, "RequestTooSoon"); // Check error code
      } else {
        assert.fail("Expected RequestTooSoon error code, but it was not found");
      }
    }
  });

  it("Admin can change the interval period", async () => {
    console.log("Admin changing the interval period...");

    const newIntervalPeriod = new anchor.BN(60);

    try {
      await program.methods.setIntervalPeriod(newIntervalPeriod)
        .accounts({
          faucet: faucetAccount.publicKey,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();
      console.log("Interval period changed successfully");
    } catch (err) {
      console.error("Error during setIntervalPeriod:", err);
      throw err;
    }

    const faucetData = await program.account.faucet.fetch(faucetAccount.publicKey);
    assert.equal(faucetData.intervalPeriod.toNumber(), 60);
  });

  it("Non-admin cannot change the interval period", async () => {
    console.log("Non-admin attempting to change the interval period...");

    const newIntervalPeriod = new anchor.BN(120);

    try {
      await program.methods.setIntervalPeriod(newIntervalPeriod)
        .accounts({
          faucet: faucetAccount.publicKey,
          admin: admin.publicKey, // Ensures that the admin field in the faucet matches the public key of the provided admin
          systemProgram: SystemProgram.programId,
        })
        .signers([userAccount]) // The non-admin user is attempting to make this change
        .rpc();
      assert.fail("Non-admin should not be able to change the interval period");
    } catch (err) {
      console.log("Expected error for non-admin attempting to change interval period:", err.message);
      if (err.error && err.error.errorCode) {
        assert.equal(err.error.errorCode.code, "Unauthorized");
      } else {
        assert.fail("Expected Unauthorized error code, but it was not found");
      }
    }
  });
});
