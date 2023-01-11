import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import {
  Account,
  createAssociatedTokenAccount,
  getAssociatedTokenAddress,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  mintToChecked,
} from "@solana/spl-token";
import { SeahorseAuction } from "../target/types/seahorse_auction";
import { airdrop, mintNewToken } from "./helpers";
const { log } = console;
describe("seahorse_auction", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const kp = () => anchor.web3.Keypair.generate();
  const pk = (key: string) => new anchor.web3.PublicKey(key);
  const spk = (key: anchor.web3.PublicKey) => key.toBase58();
  const program = anchor.workspace.SeahorseAuction as Program<SeahorseAuction>;
  const con = new anchor.web3.Connection("http://127.0.0.1:8899");
  const seller = kp(); // sellers signer
  const sellerPublicKey = seller.publicKey;
  const itemMint = kp();
  const paymentToken = kp();
  let sellerItemATA: anchor.web3.PublicKey;
  let sellerPaymentATA: anchor.web3.PublicKey;
  let auctionPDA: anchor.web3.PublicKey;
  let currencyHolder: anchor.web3.PublicKey;
  let itemHolder: anchor.web3.PublicKey;
  log("itemMint: ", spk(itemMint.publicKey));
  log("sellerPublicKey: ", spk(sellerPublicKey));
  log("paymentToken: ", spk(paymentToken.publicKey));
  // const payer = kp()
  before(async () => {
    // create a new spl token for payment
    // create a new nft for item
    await airdrop(sellerPublicKey);
    log("airdropped");
    // let mintPubkey = await createMint(
    //   connection, // conneciton
    //   feePayer, // fee payer
    //   alice.publicKey, // mint authority
    //   alice.publicKey, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
    //   8 // decimals
    // );
    // sellerItemATA = await createAssociatedTokenAccount(
    //   con,
    //   seller,
    //   itemMint.publicKey,
    //   seller.publicKey
    // );
    // log("here");
    // const mintTxn = await mintToChecked(
    //   con,
    //   seller,
    //   itemMint.publicKey,
    //   sellerItemATA,
    //   seller,
    //   1,
    //   1
    // );
    let [mint, ata, txn] = await mintNewToken(
      seller,
      sellerPublicKey,
      0,
      1,
      itemMint
    );
    sellerItemATA = ata as anchor.web3.PublicKey;
    log("itemMint txn: ", txn);
    log("sellerItemATA: ", sellerItemATA.toBase58());

    await airdrop(sellerPublicKey);
    let [mint_, ata_, txn_] = await mintNewToken(
      seller,
      sellerPublicKey,
      6,
      1000 * 10 ** 6,
      paymentToken
    );
    log("paymentTokenTxn txn: ", txn_);
    sellerPaymentATA = ata_ as anchor.web3.PublicKey;
    log("sellerPaymentATA: ", sellerPaymentATA.toBase58());
  });
  it("Is initialized!", async () => {
    // Add your test here.
    const [auctionPDA_, _] = await anchor.web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode("auction"), sellerPublicKey.toBytes()],
      program.programId
    );
    auctionPDA = auctionPDA_;
    currencyHolder = await getAssociatedTokenAddress(
      paymentToken.publicKey,
      auctionPDA_,
      true
    );
    itemHolder = await getAssociatedTokenAddress(
      itemMint.publicKey,
      auctionPDA_,
      true
    );
    const tx = await program.methods
      .createAuction(
        new anchor.BN(50),
        false,
        new anchor.BN(Date.now()),
        new anchor.BN(Date.now())
      )
      .accounts({
        currency: paymentToken.publicKey,
        currencyHolder: currencyHolder,
        item: itemMint.publicKey,
        itemHolder: itemHolder,
        seller: sellerPublicKey,
        auction: auctionPDA_,
        payer: sellerPublicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([seller])
      .rpc();
    console.log("Your transaction signature", tx);
  });
});
