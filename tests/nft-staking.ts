import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NftStaking } from "../target/types/nft_staking";
import {
    MPL_TOKEN_METADATA_PROGRAM_ID
} from "@metaplex-foundation/mpl-token-metadata";
import {
    toWeb3JsPublicKey,
} from "@metaplex-foundation/umi-web3js-adapters";
import { publicKey } from "@metaplex-foundation/umi";
import { expect } from "chai";
import { NftMint, setupNft } from "../utils/setupNft";
import { PublicKey } from "@solana/web3.js";
const { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } = require('@solana/spl-token');
const customEndPoint = "https://bold-old-snowflake.solana-devnet.quiknode.pro/fb01c0a0fd61fa6ff2092476e9e30c2fb434cb5c/";

describe("stake test", () => {
    // Configure the client to use the local cluster.
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.NftStaking as Program<NftStaking>;
    const wallet = anchor.workspace.NftStaking.provider.wallet;

    const nftMintAddress = new PublicKey("1xyGvMGVHUXqhHjCQqCsDW8XGDUWJuuEuopEFsEp3aS");

    const metadataProgramId = toWeb3JsPublicKey(MPL_TOKEN_METADATA_PROGRAM_ID);
    const [nftHolderAccount] = PublicKey.findProgramAddressSync([wallet.publicKey.toBytes(), TOKEN_PROGRAM_ID.toBuffer(), nftMintAddress.toBuffer()], ASSOCIATED_TOKEN_PROGRAM_ID);
    const [nftEdition] = PublicKey.findProgramAddressSync([Buffer.from('metadata'),metadataProgramId.toBuffer(),nftMintAddress.toBuffer(),Buffer.from('edition')], metadataProgramId);
    const [metadataAccount] = PublicKey.findProgramAddressSync([Buffer.from('metadata'), metadataProgramId.toBuffer(),nftMintAddress.toBuffer()], metadataProgramId);
    const [stakeState] = PublicKey.findProgramAddressSync([wallet.publicKey.toBuffer(), nftHolderAccount.toBuffer()], program.programId);
    const [programAuthority] = PublicKey.findProgramAddressSync([Buffer.from("authority")], program.programId);
    const [stakeStatePda] = PublicKey.findProgramAddressSync([wallet.payer.publicKey.toBuffer(), nftHolderAccount.toBuffer()],program.programId);

    // it("mint", async () => {
    //     await setupNft(program, wallet.payer, customEndPoint);
    // });

    it("stake", async () => {
        console.log("Initialising stakes");
        await program.methods
        .stake(0)
        .accounts({
            nftTokenAccount: nftHolderAccount,
            nftMint: nftMintAddress,
            nftEdition: nftEdition,
            stakeState: stakeState,
            programAuthority: programAuthority,
            tokenMetadataAccount: metadataAccount,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            metadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
        })
        .rpc();

        const account = await program.account.userStakeInfo.fetch(stakeStatePda);
        program.account.userStakeInfo.idlAccount.name;
        console.log("stake status after staking: ", account);
        expect(account.stakeState).to.have.property("staked");
    });


    it("Unstakes", async () => {
        await program.methods
        .unstake()
        .accounts({
            nftTokenAccount: nftHolderAccount,
            nftMint: nftMintAddress,
            nftEdition: nftEdition,
            metadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
            stakeState: stakeState,
            programAuthority: programAuthority,
        })
        .rpc();
    
        const stakeAccount = await program.account.userStakeInfo.fetch(stakeStatePda);
        console.log("stake status after unstake: ", stakeAccount);
    });
});
