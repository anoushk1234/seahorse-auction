import * as anchor from "@project-serum/anchor";
import {
  createAssociatedTokenAccount,
  createMint,
  mintToChecked,
} from "@solana/spl-token";
// import { TOKEN_PROGRAM_ID } from "@project-serum/anchor/dist/cjs/utils/token";
import { PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";
const { log } = console;
const provider = anchor.getProvider();

export async function airdrop(key: PublicKey) {
  // const connection = new anchor.web3.Connection("http://127.0.0.1:8899");
  const airdropSig = await provider.connection.requestAirdrop(
    key,
    10 * LAMPORTS_PER_SOL
  );
  return provider.connection.confirmTransaction(airdropSig, "finalized");
}

export async function getBalance(key: PublicKey) {
  console.log(provider.connection.rpcEndpoint);
  return await provider.connection.getBalance(key);
}

export async function mintNewToken(
  payer: anchor.web3.Signer,
  auth: anchor.web3.PublicKey,
  decimals: number,
  amt: number,
  keypair: anchor.web3.Keypair
) {
  const mintPubkey = await createMint(
    provider.connection, // conneciton
    payer, // fee payer
    auth, // mint authority
    auth, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
    decimals, // decimals
    keypair
  );
  const ata = await createAssociatedTokenAccount(
    provider.connection,
    payer,
    mintPubkey,
    auth
  );
  // log("here");
  const mintTxn = await mintToChecked(
    provider.connection,
    payer,
    mintPubkey,
    ata,
    auth,
    amt,
    decimals
  );
  return [mintPubkey, ata, mintTxn];
}
