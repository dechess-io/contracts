use anchor_lang::prelude::*;
use chess::{ Board, ChessMove, Color, Game, Square };
use std::str::FromStr;

declare_id!("4YJdg3btfUVYP6PZsGLhaJMLaBxFzmM1MWvkS2BYKHPi");

#[program]
pub mod contracts {
    use chess::{ BoardStatus, MoveGen };

    use super::*;

    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        player_1: Pubkey,
        player_2: Pubkey
    ) -> Result<()> {
        let game = &mut ctx.accounts.game;

        game.admin = *ctx.accounts.admin.key;
        game.player_1 = player_1;
        game.player_2 = player_2;

        game.board = Board::default().to_string();
        game.turn = player_1;

        game.state = GameState::Ongoing;

        Ok(())
    }

    pub fn make_move(ctx: Context<MakeMove>, from: String, to: String) -> Result<()> {
        let game_account = &mut ctx.accounts.game;

        // Ensure the game is ongoing
        require!(game_account.state == GameState::Ongoing, ChessError::GameFinished);

        // Ensure that only the admin can make moves
        require!(ctx.accounts.admin.key() == game_account.admin, ChessError::Unauthorized);

        // Check if it's the correct player's turn (alternating between player_1 and player_2)
        if game_account.turn == game_account.player_1 {
            // Ensure it's player_1's turn (White's move)
            println!("It's player_1's turn (White).");
        } else if game_account.turn == game_account.player_2 {
            // Ensure it's player_2's turn (Black's move)
            println!("It's player_2's turn (Black).");
        } else {
            return Err(ChessError::InvalidMove.into());
        }

        // Parse the current board from the FEN string stored in the game account
        let mut current_board = Board::from_str(&game_account.board).map_err(
            |_| ChessError::InvalidMove
        )?; // Handle the result properly with map_err

        // Parse the from and to squares
        let from_square = Square::from_str(&from).map_err(|_| ChessError::InvalidMove)?; // Handle error if the square is invalid
        let to_square = Square::from_str(&to).map_err(|_| ChessError::InvalidMove)?; // Handle error if the square is invalid
        let chess_move = ChessMove::new(from_square, to_square, None);

        // Check if the move is legal by generating the legal moves
        let legal_moves = MoveGen::new_legal(&current_board);
        if !legal_moves.into_iter().any(|m| m == chess_move) {
            return Err(ChessError::InvalidMove.into()); // Return an error if the move is illegal
        }

        // Apply the move to the board
        current_board = current_board.make_move_new(chess_move);

        // Update the game account's board with the new FEN string after the move
        game_account.board = current_board.to_string();

        // Check the game result after the move
        let current_status = current_board.status();
        if current_status == BoardStatus::Checkmate {
            // If it's player_1's turn and there's a checkmate, player_1 wins
            if game_account.turn == game_account.player_1 {
                game_account.state = GameState::WhiteWon;
                game_account.winner = Some(game_account.player_1);
            } else {
                game_account.state = GameState::BlackWon;
                game_account.winner = Some(game_account.player_2);
            }
        } else if current_status == BoardStatus::Stalemate {
            // If it's a draw, update the state to Draw
            game_account.state = GameState::Draw;
            game_account.winner = None; // No winner
        } else {
            // If the game is not over, switch turns
            if game_account.turn == game_account.player_1 {
                game_account.turn = game_account.player_2; // Switch to player_2 (Black)
            } else {
                game_account.turn = game_account.player_1; // Switch to player_1 (White)
            }
        }

        // Return Ok(()) to signify success
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(init, payer = admin, space = 240)]
    pub game: Account<'info, GameAccount>,
    #[account(mut)]
    pub admin: Signer<'info>, // Admin initializes the game
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MakeMove<'info> {
    #[account(mut)]
    pub game: Account<'info, GameAccount>,
    #[account(signer)]
    pub admin: Signer<'info>, // Only the admin can make moves
}

#[account]
pub struct GameAccount {
    pub admin: Pubkey, // Admin who created the game
    pub player_1: Pubkey, // Player 1 (White pieces)
    pub player_2: Pubkey, // Player 2 (Black pieces)
    pub board: String, // FEN string representing the chessboard state
    pub turn: Pubkey, // Whose turn it is (Player 1 starts)
    pub state: GameState, // Current state of the game
    pub winner: Option<Pubkey>, // Winner of the game (None if no winner yet)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum GameState {
    Ongoing,
    Draw,
    WhiteWon,
    BlackWon,
    Canceled, // If the game is canceled before starting
}

#[error_code]
pub enum ChessError {
    #[msg("The game is already finished.")]
    GameFinished,
    #[msg("Only the admin can make moves.")]
    Unauthorized,
    #[msg("Invalid move.")]
    InvalidMove,
}
