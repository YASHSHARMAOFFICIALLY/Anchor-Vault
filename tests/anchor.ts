import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { AnchorVault } from "../target/types/anchor_vault";
import { Commitment,LAMPORTS_PER_SOL,PublicKey,SystemProgram } from "@solana/web3.js";
import NodeWallet from "@anchor-lang/core/dist/cjs/nodewallet";
import { BN } from "bn.js";
import { expect } from "chai";

const commitement:Commitment = "confirmed";

describe("anchor-vault",()=>{

    const confirmTx = async (signature:String)=>{
        const latestBlockhash = await anchor.getProvider().connection.getLatestBlockhash();

        await anchor.getProvider().connection.confirmTransaction(
            {
                signature,
                ...latestBlockhash
            },
            commitement
        );
    };

    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider)

    const program = anchor.workspace.anchor_vault as Program<AnchorVault>;

    const user = provider.wallet.publicKey;

    //Derive Pdas

    const [vaultPda,vaultBump] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"),vaultStatePda.toBuffer()],
        program.programId
    )

    before(async ()=>{
        const sig = await provider.connection.requestAirdrop(user,10*LAMPORTS_PER_SOL);
        await confirmTx(sig);
    })

    it("Initialise the vault",async()=>{

        const tx = await program.methods.initialize().accountsStrict({
            user:user,
            vaultState:vaultStatepda,
            vault:vaultPda,
            systemProgram:SystemProgram.programId
        })
        .rpc();

        await confirmTx(tx)

        const vaultState = await program.vaultState.fetch(vaulteStatePda);
        expect(vaultState.vaultBump).to.equal(vaultBump)
        expect(vaultState.stateBump).to.equal.apply(stateBump)
    })

})