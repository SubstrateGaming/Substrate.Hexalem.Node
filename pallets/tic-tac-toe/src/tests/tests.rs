use crate::{tests::mock::*, *};
use frame_support::{assert_noop, assert_ok};

mod create {
	use super::*;

	#[test]
	fn can_create_game() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));

			run_to_block(2);

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(PendingGames::<Test>::contains_key(0));
		});
	}

	#[test]
	fn different_accounts_can_create_different_games() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(PendingGames::<Test>::contains_key(0));

			run_to_block(2);

			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(BOB)));

			assert!(ActivePlayers::<Test>::contains_key(BOB));
			assert!(PendingGames::<Test>::contains_key(1));
		});
	}

	#[test]
	fn cannot_create_mutliple_games_with_signle_account() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(PendingGames::<Test>::contains_key(0));

			run_to_block(2);

			assert_noop!(
				TicTacToe::create_game(RuntimeOrigin::signed(ALICE)),
				Error::<Test>::CannotCreateNewGame
			);
		});
	}
}

mod join {
	use super::*;

	#[test]
	fn can_join_game() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(PendingGames::<Test>::contains_key(0));

			run_to_block(2);

			assert_ok!(TicTacToe::join_pending_game(RuntimeOrigin::signed(BOB)));

			assert!(ActivePlayers::<Test>::contains_key(BOB));

			assert!(!PendingGames::<Test>::contains_key(0));
			assert!(ActiveGames::<Test>::contains_key(0));
		});
	}

	#[test]
	fn cannot_join_game_if_host() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(PendingGames::<Test>::contains_key(0));

			run_to_block(2);

			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(BOB)));

			assert!(ActivePlayers::<Test>::contains_key(BOB));
			assert!(PendingGames::<Test>::contains_key(1));

			assert_noop!(
				TicTacToe::join_pending_game(RuntimeOrigin::signed(BOB)),
				Error::<Test>::CannotJoinAnotherGame
			);
		});
	}

	#[test]
	fn cannot_join_own_game() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(PendingGames::<Test>::contains_key(0));

			run_to_block(2);

			assert_noop!(
				TicTacToe::join_pending_game(RuntimeOrigin::signed(ALICE)),
				Error::<Test>::CannotJoinAnotherGame
			);
		});
	}

	#[test]
	fn cannot_join_started_game() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));
			assert_ok!(TicTacToe::join_pending_game(RuntimeOrigin::signed(BOB)));

			run_to_block(2);

			assert_noop!(
				TicTacToe::join_pending_game(RuntimeOrigin::signed(CHARLIE)),
				Error::<Test>::NoPendingGamesFound
			);
		});
	}
}
