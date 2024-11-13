import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { UniversalLiquidityPoolSoonv1 } from "../target/types/universal_liquidity_pool_soonv1";
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Keypair } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress, createAssociatedTokenAccount } from "@solana/spl-token";

describe("Universal Liquidity Pool Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.UniversalLiquidityPoolSoonv1 as Program<UniversalLiquidityPoolSoonv1>;
      const wallet = provider.wallet as anchor.Wallet;

  // Token existente
  const tokenMint = new PublicKey("Ff8cYzTU8o6LV68TgzaRNZhNFTvt7kwW7bn5PnxH1zcx");
  
  let factoryPDA: PublicKey;
  let factoryBump: number;
  let poolPDA: PublicKey;
  let poolBump: number;
  let lpTokenMint: Keypair;
  let poolTokenAccount: PublicKey;
  let userTokenAccount: PublicKey;
  let userLpTokenAccount: PublicKey;

  before(async () => {
    // Encontrar PDA para factory
    [factoryPDA, factoryBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("factory")],
      program.programId
    );

    // Encontrar PDA para pool
    [poolPDA, poolBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), tokenMint.toBuffer()],
      program.programId
    );

    // Crear keypair para LP token mint
    lpTokenMint = Keypair.generate();

    // Obtener cuenta de token del usuario
    userTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      wallet.publicKey
    );

    // Crear cuenta para LP tokens del usuario
    userLpTokenAccount = await getAssociatedTokenAddress(
      lpTokenMint.publicKey,
      wallet.publicKey
    );

    // Obtener cuenta de token del pool
    poolTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      poolPDA,
      true
    );
  });

  it("Initializes the factory", async () => {
    try {
      const accounts = {
        authority: wallet.publicKey,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      };

      await program.methods
        .initializeFactory()
        .accounts(accounts)
        .rpc();
      
      console.log("Factory initialized");
    } catch (error) {
      console.error("Error:", error);
      throw error;
    }
  });

  it("Creates a new pool", async () => {
    try {
      const accounts = {
        authority: wallet.publicKey,
        tokenMint: tokenMint,
        lpTokenMint: lpTokenMint.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      };

      await program.methods
        .createPool(poolBump)
        .accounts(accounts)
        .signers([lpTokenMint])
        .rpc();

      console.log("Pool created");
    } catch (error) {
      console.error("Error:", error);
      throw error;
    }
  });

  it("Adds liquidity to the pool", async () => {
    const amountSol = new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL);
    const amountToken = new anchor.BN(1000_000000000);

    try {
      const accounts = {
        user: wallet.publicKey,
        userTokenAccount: userTokenAccount,
        userLpTokenAccount: userLpTokenAccount,
        poolTokenAccount: poolTokenAccount,
        lpTokenMint: lpTokenMint.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      };

      await program.methods
        .addLiquidity(amountSol, amountToken)
        .accounts(accounts)
        .rpc();

      console.log("Liquidity added");
    } catch (error) {
      console.error("Error:", error);
      throw error;
    }
  });

  it("Removes liquidity from the pool", async () => {
    const lpAmount = new anchor.BN(50_000000);

    try {
      const accounts = {
        user: wallet.publicKey,
        userTokenAccount: userTokenAccount,
        userLpTokenAccount: userLpTokenAccount,
        poolTokenAccount: poolTokenAccount,
        lpTokenMint: lpTokenMint.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      };

      await program.methods
        .removeLiquidity(lpAmount)
        .accounts(accounts)
        .rpc();

      console.log("Liquidity removed");
    } catch (error) {
      console.error("Error:", error);
      throw error;
    }
  });
});
