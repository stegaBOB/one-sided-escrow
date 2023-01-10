import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { utf8 } from "@project-serum/anchor/dist/cjs/utils/bytes";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";
import { Connection, LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";
const PROGRAM_ID = "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS";

const convertLamports = (num: number) => {
  return num / LAMPORTS_PER_SOL;


}

describe("one-sided-escrow", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.OneSidedEscrow

  const buyer = anchor.web3.Keypair.generate();
  const seller = anchor.web3.Keypair.generate();
  const escrow: PublicKey = findProgramAddressSync(
    [utf8.encode('escrow'), buyer.publicKey.toBuffer(), seller.publicKey.toBuffer()],
    new PublicKey(PROGRAM_ID)
  )[0];

  const STARTING_LAMPORTS = 10000000000;
  const TO_TRANSFER = 1000000000

  it("Is initialized!", async () => {

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(provider.publicKey, STARTING_LAMPORTS),
      "confirmed"
    );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(buyer.publicKey, STARTING_LAMPORTS),
      "confirmed"
    );

    console.log(await provider.connection.getBalance(buyer.publicKey) / LAMPORTS_PER_SOL);

    //test the createEscrow method
    await program.methods.createEscrow(seller.publicKey, new anchor.BN(TO_TRANSFER
    )).accounts({
      payer: buyer.publicKey,
      buyer: buyer.publicKey,
      escrow: escrow,
      systemProgram: SystemProgram.programId,
    }).signers([buyer]).rpc()

    const buyerAmount = (await provider.connection.getBalance(buyer.publicKey) / LAMPORTS_PER_SOL);
    const escrowAmount = (await provider.connection.getBalance(escrow) / LAMPORTS_PER_SOL);

    const escrowAccount = await program.account.escrow.fetch(escrow);
    assert.equal(buyerAmount, convertLamports(STARTING_LAMPORTS - TO_TRANSFER));
    assert.equal(escrowAmount, convertLamports(TO_TRANSFER));
    assert.ok(escrowAccount.buyer.equals(buyer.publicKey));
    assert.ok(escrowAccount.seller.equals(seller.publicKey));


    let sellerAmount = (await provider.connection.getBalance(seller.publicKey) / LAMPORTS_PER_SOL);

    await program.methods.completeSale().accounts({
      buyer: buyer.publicKey,
      seller: seller.publicKey,
      escrow: escrow,
    }).signers([buyer]).rpc()


    sellerAmount = (await provider.connection.getBalance(seller.publicKey) / LAMPORTS_PER_SOL);

    assert.equal(sellerAmount, convertLamports(TO_TRANSFER));

  });

});


