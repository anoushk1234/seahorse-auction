import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import {
  Account,
  createAssociatedTokenAccount,
  getAssociatedTokenAddress,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  mintToChecked,
  transferChecked,
} from "@solana/spl-token";
import { SeahorseAuction } from "../target/types/seahorse_auction";
import { airdrop, mintNewToken, pdaTokenAccount } from "./helpers";
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
  const bidder = kp();
  let sellerItemATA: anchor.web3.PublicKey;
  let sellerPaymentATA: anchor.web3.PublicKey;
  let auctionPDA: anchor.web3.PublicKey;
  let currencyHolder: anchor.web3.PublicKey;
  let itemHolder: anchor.web3.PublicKey;
  let bidderPaymentATA: anchor.web3.PublicKey;
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
  it("It creates an auction", async () => {
    // Add your test here.
    const [auctionPDA_, _] = await anchor.web3.PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode("auction"), sellerPublicKey.toBytes()],
      program.programId
    );
    auctionPDA = auctionPDA_;
    currencyHolder = await pdaTokenAccount(
      [
        anchor.utils.bytes.utf8.encode("currency_holder"),
        seller.publicKey.toBytes(),
        paymentToken.publicKey.toBytes(),
      ],
      program
    );

    itemHolder = await pdaTokenAccount(
      [
        anchor.utils.bytes.utf8.encode("item_holder"),
        seller.publicKey.toBytes(),
        itemMint.publicKey.toBytes(),
      ],
      program
    );
    log("auctionPDA: ", auctionPDA.toBase58());
    log("currencyHolder: ", currencyHolder.toBase58());
    log("itemHolder: ", itemHolder.toBase58());

    // const transferItemFromSellerToItemHolder = await transferChecked(
    //   con,
    //   seller,
    //   sellerItemATA,
    //   itemMint.publicKey,
    //   itemHolder,
    //   seller,
    //   1,
    //   0
    // );
    // log(
    //   "transferItemFromSellerToItemHolder: ",
    //   transferItemFromSellerToItemHolder
    // );
    const tx = await program.methods
      .createAuction(
        new anchor.BN(50 * 10 ** 6),
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
  it("It bids on an auction", async () => {
    const pre = await program.methods
      .depositItem()
      .accounts({
        sellerItemAta: sellerItemATA,
        itemHolder: itemHolder,
        payer: sellerPublicKey,
      })
      .signers([seller])
      .rpc({
        skipPreflight: true,
      });
    console.log("Your transaction signature", pre);
    const bidderPublicKey = bidder.publicKey;
    await airdrop(bidderPublicKey);
    const price = new anchor.BN(80 * 10 ** 6);
    bidderPaymentATA = await createAssociatedTokenAccount(
      con,
      bidder,
      paymentToken.publicKey,
      bidderPublicKey
    );
    const mintTxn = await transferChecked(
      con,
      bidder,
      sellerPaymentATA,
      paymentToken.publicKey,
      bidderPaymentATA,
      seller,
      200 * 10 ** 6,
      6
    );
    log(
      "bid auction price: ",
      await (
        await program.account.auction.fetch(auctionPDA)
      ).refundReceiver.toBase58(),
      price.toNumber(),
      bidderPaymentATA.toBase58()
    );
    const tx = await program.methods
      .bid(price)
      .accounts({
        refundReceiver: currencyHolder, // first bid pass currency holder, second bid pass previous bidder
        currencyHolder: currencyHolder,
        bidder: bidderPaymentATA,
        auction: auctionPDA,
        authority: bidderPublicKey,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      })
      .signers([bidder])
      .rpc({
        skipPreflight: true,
      });
    console.log("Your transaction signature", tx);
  });
  it("It ends an auction", async () => {
    const itemHolderForBidder = await createAssociatedTokenAccount(
      con,
      bidder,
      itemMint.publicKey,
      bidder.publicKey
    );
    const tx = await program.methods
      .closeAuction()
      .accounts({
        auction: auctionPDA,
        currencyHolder: currencyHolder,
        itemHolder: itemHolder,
        itemReceiver: itemHolderForBidder,
        seller: sellerPublicKey,
        sellerAta: sellerPaymentATA,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      })
      .signers([seller])
      .rpc({
        skipPreflight: true,
      });
    console.log("Your transaction signature", tx);
  });
});
