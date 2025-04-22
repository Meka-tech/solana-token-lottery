import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";
import { Tokenlottery } from "../target/types/tokenlottery";

describe("tokenlottery", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = provider.wallet as anchor.Wallet;

  const wallet = provider.wallet as anchor.Wallet;

  const program = anchor.workspace.Tokenlottery as Program<Tokenlottery>;

  const tokenlotteryKeypair = Keypair.generate();

  it("should init config", async () => {
    const InitConfigIx = await program.methods
      .initializeConfig(
        new anchor.BN(0),
        new anchor.BN(1845250158),
        new anchor.BN(1000)
      )
      .instruction();

    const blockhashWithContext = await provider.connection.getLatestBlockhash();

    const tx = new anchor.web3.Transaction({
      feePayer: wallet.publicKey,
      blockhash: blockhashWithContext.blockhash,
      lastValidBlockHeight: blockhashWithContext.lastValidBlockHeight
    }).add(InitConfigIx);

    const signature = await anchor.web3.sendAndConfirmTransaction(
      provider.connection,
      tx,
      [wallet.payer],
      { skipPreflight: true }
    );

    console.log("Your transaction signature", signature);
  });
});
