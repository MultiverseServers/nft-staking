import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NftStaking } from "../target/types/nft_staking";
import {
    MPL_TOKEN_METADATA_PROGRAM_ID,
} from "@metaplex-foundation/mpl-token-metadata";
import {
    toWeb3JsPublicKey,
} from "@metaplex-foundation/umi-web3js-adapters";
import { publicKey } from "@metaplex-foundation/umi";
import { expect } from "chai";
import { NftMint, setupNft } from "../utils/setupNft";

describe("stake test", () => {
    // Configure the client to use the local cluster.
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.NftStaking as Program<NftStaking>;
    const wallet = anchor.workspace.NftStaking.provider.wallet;

    const mintAddress = publicKey("9y9GSY23pM2sv5pW5FoQUPYvQdACEQMH5LACpuEkkZum");
    const owner = publicKey("687sDBgayiTB3MQPqbhEdhh3ZzEmNGKvVLcwaP8YhZGJ");
    const nftEditionKey = publicKey("6CxJmzf3LPm8TCd3op7CrfE7c82Fb5nagHm5BPkt7jUh");

    // We convert the public keys to web3.js format because anchor uses web3.js
    const nftMintAddress = toWeb3JsPublicKey(mintAddress);
    const nftAccount = toWeb3JsPublicKey(owner)
    const nftEdition = toWeb3JsPublicKey(nftEditionKey);

    const [stakeState] = anchor.web3.PublicKey.findProgramAddressSync([wallet.publicKey.toBuffer(), nftAccount.toBuffer()], program.programId);
    const [programAuthority] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("authority")], program.programId);
    const [stakeStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [wallet.payer.publicKey.toBuffer(), nftAccount.toBuffer()],
        program.programId
    );

    // it("mint", async () => {
    //     await setupNft(program, wallet.payer, "https://bold-old-snowflake.solana-devnet.quiknode.pro/fb01c0a0fd61fa6ff2092476e9e30c2fb434cb5c/");
    // });

    it("stake", async () => {
        console.log("Initialising stakes");
        await program.methods
            .stake()
            .accounts({
                nftTokenAccount: nftAccount,
                nftMint: nftMintAddress,
                nftEdition: nftEdition,
                metadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
                stakeState: stakeState,
                programAuthority: programAuthority,
            })
            .rpc();

        const account = await program.account.userStakeInfo.fetch(stakeStatePda);
        console.log("stake status after staking: ", account);
        expect(account.stakeState).to.have.property("staked");
  });


    // it("Unstakes", async () => {
    //     await program.methods
    //     .unstake()
    //     .accounts({
    //         nftTokenAccount: nftAccount,
    //         nftMint: nftMintAddress,
    //         nftEdition: nftEdition,
    //         metadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
    //         stakeState: stakeState,
    //         programAuthority: programAuthority,
    //     })
    //     .rpc();
    //
    //     const stakeAccount = await program.account.userStakeInfo.fetch(stakeStatePda);
    //     console.log("stake status after unstake: ", stakeAccount);
    // });
});
