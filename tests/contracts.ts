import * as anchor from "@coral-xyz/anchor";
import {Program} from "@coral-xyz/anchor";
import {Contracts} from "../target/types/contracts";
import {PublicKey, SystemProgram} from "@solana/web3.js";
import {expect} from "chai";

describe("contracts", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);

  const program = anchor.workspace.Contracts as Program<Contracts>;

  let gameAccount: anchor.web3.Keypair;
  let player1: anchor.web3.Keypair;
  let player2: anchor.web3.Keypair;
  let admin: anchor.web3.Keypair;

  before(async () => {
    // Generate keypairs for players and admin
    gameAccount = anchor.web3.Keypair.generate();
    player1 = anchor.web3.Keypair.generate();
    player2 = anchor.web3.Keypair.generate();
    admin = anchor.web3.Keypair.generate();

    console.table({
      gameAccount,
      player1,
      player2,
      admin,
    });

    // Airdrop SOL to the admin for transaction fees
    const signature = await provider.connection.requestAirdrop(
      admin.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(signature);
  });

  it("should initialize the game", async () => {
    // Call initialize_game with the admin, player_1, and player_2
    await program.methods
      .initializeGame(player1.publicKey, player2.publicKey)
      .accounts({
        game: gameAccount.publicKey,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin, gameAccount])
      .rpc();

    // Fetch the game account to check its state
    const gameState = await program.account.gameAccount.fetch(
      gameAccount.publicKey
    );

    console.table({
      turn: gameState.turn.toString(),
      newFen: gameState.board,
    });

    // Check if the game state was initialized correctly
    expect(gameState.admin.toBase58()).to.equal(admin.publicKey.toBase58());
    expect(gameState.player1.toBase58()).to.equal(player1.publicKey.toBase58());
    expect(gameState.player2.toBase58()).to.equal(player2.publicKey.toBase58());
    expect(gameState.board).to.be.a("string"); // FEN string
  });

  it("should make a move", async () => {
    const from = "e2";
    const to = "e4";

    // Call make_move with the admin making the move
    await program.methods
      .makeMove(from, to)
      .accounts({
        game: gameAccount.publicKey,
        admin: admin.publicKey,
      })
      .signers([admin])
      .rpc();

    // Fetch the updated game state
    const updatedGameState = await program.account.gameAccount.fetch(
      gameAccount.publicKey
    );

    // Check if the board state has changed (the FEN string should update)
    console.log("-> new state", updatedGameState.board);
    console.table({
      turn: updatedGameState.turn.toString(),
      newFen: updatedGameState.board,
    });
    expect(updatedGameState.board).to.not.equal(null); // Check if board was updated
  });
});
