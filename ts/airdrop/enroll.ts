import {
    address,
    appendTransactionMessageInstructions,
    assertIsTransactionWithinSizeLimit,
    createKeyPairSignerFromBytes,
    createSolanaRpc,
    createSolanaRpcSubscriptions,
    createTransactionMessage,
    devnet,
    getSignatureFromTransaction,
    pipe,
    sendAndConfirmTransactionFactory,
    setTransactionMessageFeePayerSigner,
    setTransactionMessageLifetimeUsingBlockhash,
    signTransactionMessageWithSigners,
    addSignersToTransactionMessage,
    getProgramDerivedAddress,
    generateKeyPairSigner,
    getAddressEncoder
  } from "@solana/kit";
  import { getInitializeInstruction, getSubmitTsInstruction } from "../clients/js/src/generated";
  import fs from "fs";

  const MPL_CORE_PROGRAM = address("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");
  const PROGRAM_ADDRESS = address("TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM");
  const SYSTEM_PROGRAM = address("11111111111111111111111111111111");
  const COLLECTION = address("5ebsp5RChCGK7ssRZMVMufgVZhd2kFbNaotcZ5UvytN2");


  async function main() {
    const walletFile = fs.readFileSync("./Turbin3-wallet.json", "utf-8");
    const walletData = JSON.parse(walletFile);
    const keypair = await createKeyPairSignerFromBytes(new Uint8Array(walletData));

    console.log("Wallet address:", keypair.address);

    const HELIUS_API_KEY = "ad67ac5f-ba98-4639-a0fc-de25acdae3fa";
    const rpc = createSolanaRpc(devnet(`https://devnet.helius-rpc.com/?api-key=${HELIUS_API_KEY}`));
    const rpcSubscriptions = createSolanaRpcSubscriptions(
      devnet(`wss://devnet.helius-rpc.com/?api-key=${HELIUS_API_KEY}`)
    );

    const addressEncoder = getAddressEncoder();

    const accountSeeds = [
      Buffer.from("prereqs"),
      addressEncoder.encode(keypair.address)
    ];
    const [account, _bump] = await getProgramDerivedAddress({
      programAddress: PROGRAM_ADDRESS,
      seeds: accountSeeds
    });

    console.log("Account PDA:", account);

    const mintKeyPair = await generateKeyPairSigner();
    console.log("Mint address:", mintKeyPair.address);

    const authoritySeeds = [
      Buffer.from("collection"),
      addressEncoder.encode(COLLECTION)
    ];
    const [authority, _authorityBump] = await getProgramDerivedAddress({
      programAddress: PROGRAM_ADDRESS,
      seeds: authoritySeeds
    });
    console.log("Authority PDA:", authority);
    console.log("Collection address:", COLLECTION);

    const sendAndConfirmTransaction = sendAndConfirmTransactionFactory({
      rpc,
      rpcSubscriptions
    });

    const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

    console.log("\n=== Checking Enrollment Status ===");

    try {
      const accountInfo = await rpc.getAccountInfo(account).send();
      if (accountInfo.value && accountInfo.value.owner === PROGRAM_ADDRESS) {
        console.log("✅ Enrollment account already exists! Skipping initialization.");
      } else {
        console.log("Account exists but not owned by program - reinitializing...");
        throw new Error("Account exists but not owned by program");
      }
    } catch (error) {
      console.log("🔄 Account doesn't exist or check failed, proceeding with initialization...");

      const initializeIx = getInitializeInstruction({
        github: "yashvikram30",
        user: keypair,
        account,
        systemProgram: SYSTEM_PROGRAM
      });

      const transactionMessageInit = pipe(
        createTransactionMessage({ version: 0 }),
        tx => setTransactionMessageFeePayerSigner(keypair, tx),
        tx => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
        tx => appendTransactionMessageInstructions([initializeIx], tx)
      );

      const signedTxInit = await signTransactionMessageWithSigners(transactionMessageInit);
      assertIsTransactionWithinSizeLimit(signedTxInit);

      try {
        const result = await sendAndConfirmTransaction(
          signedTxInit,
          { commitment: 'confirmed', skipPreflight: false }
        );
        console.log("Initialize result:", result);

        const signatureInit = getSignatureFromTransaction(signedTxInit);
        console.log(`✅ Success! Check out your TX here:`);
        console.log(`https://explorer.solana.com/tx/${signatureInit}?cluster=devnet`);
      } catch (e) {
        if (e instanceof Error) {
          console.error(`❌ Oops, something went wrong: ${e.message}`);
          console.error('Error stack:', e.stack);
        } else {
          console.error('❌ Oops, something went wrong with an unknown error:', e);
        }
        return;
      }
    }

    console.log("\n=== Executing Submit TS Transaction ===");

    const { value: latestBlockhashSubmit } = await rpc.getLatestBlockhash().send();

    const submitIx = getSubmitTsInstruction({
      user: keypair,
      account,
      mint: mintKeyPair,
      collection: COLLECTION,
      authority,
      mplCoreProgram: MPL_CORE_PROGRAM,
      systemProgram: SYSTEM_PROGRAM
    });

    const transactionMessageSubmit = pipe(
      createTransactionMessage({ version: 0 }),
      tx => setTransactionMessageFeePayerSigner(keypair, tx),
      tx => setTransactionMessageLifetimeUsingBlockhash(latestBlockhashSubmit, tx),
      tx => appendTransactionMessageInstructions([submitIx], tx),
      tx => addSignersToTransactionMessage([mintKeyPair], tx)
    );

    const signedTxSubmit = await signTransactionMessageWithSigners(transactionMessageSubmit);
    assertIsTransactionWithinSizeLimit(signedTxSubmit);

    try {
      const result = await sendAndConfirmTransaction(
        signedTxSubmit,
        { commitment: 'confirmed', skipPreflight: false }
      );
      console.log("Submit result:", result);

      const signatureSubmit = getSignatureFromTransaction(signedTxSubmit);
      console.log(`✅ Success! Check out your TX here:`);
      console.log(`https://explorer.solana.com/tx/${signatureSubmit}?cluster=devnet`);
      console.log(`\n🎉 Congratulations! You've completed the Turbin3 prerequisites!`);
    } catch (e) {
      if (e instanceof Error) {
        console.error(`❌ Oops, something went wrong: ${e.message}`);
        console.error('Error stack:', e.stack);
        const anyErr: any = e;
        if (anyErr.context) {
          console.error('RPC context:', anyErr.context);
        }
        if (anyErr.details) {
          console.error('RPC details:', anyErr.details);
        }
      } else {
        console.error('❌ Oops, something went wrong with an unknown error:', e);
      }
    }
  }
  
  main().catch(console.error);