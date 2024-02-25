use crate::{mock::*, Error};
use frame_support::StorageHasher;
use frame_support::Blake2_128;
use parity_scale_codec::Encode;

#[test]
fn test_is_queued() {
	new_test_ext().execute_with(|| {
		let player1 = 1;

		assert_eq!(MatchMaker::do_queue_size(0), 0);
		assert!(!MatchMaker::do_is_queued(player1));
		assert_eq!(MatchMaker::do_add_queue(player1, 0), Ok(()));
		assert!(MatchMaker::do_is_queued(player1));
		MatchMaker::do_empty_queue(0);
		assert!(!MatchMaker::do_is_queued(player1));
	});
}

#[test]
fn test_try_duplicate_queue() {
	new_test_ext().execute_with(|| {
		let player1 = 1;
		let player2 = 2;

		assert_eq!(MatchMaker::do_queue_size(0), 0);
		assert_eq!(MatchMaker::do_add_queue(player1, 0), Ok(()));
		// try same bracket
		assert_eq!(
			MatchMaker::do_add_queue(player1, 0),
			Err(Error::<TestRuntime>::AlreadyQueued.into())
		);
		// try other bracket
		assert_eq!(
			MatchMaker::do_add_queue(player1, 1),
			Err(Error::<TestRuntime>::AlreadyQueued.into())
		);

		assert_eq!(MatchMaker::do_add_queue(player2, 1), Ok(()));
		// try same bracket
		assert_eq!(
			MatchMaker::do_add_queue(player2, 1),
			Err(Error::<TestRuntime>::AlreadyQueued.into())
		);
		// try other bracket
		assert_eq!(
			MatchMaker::do_add_queue(player2, 0),
			Err(Error::<TestRuntime>::AlreadyQueued.into())
		);
	});
}

#[test]
fn test_add_queue() {
	new_test_ext().execute_with(|| {
		let player1 = 1;
		let player2 = 2;

		assert_eq!(MatchMaker::do_queue_size(0), 0);
		assert!(MatchMaker::do_try_match().is_empty());
		assert_eq!(MatchMaker::do_add_queue(player1, 0), Ok(()));
		assert_eq!(MatchMaker::do_queue_size(0), 1);
		assert!(MatchMaker::do_try_match().is_empty());
		assert_eq!(MatchMaker::do_add_queue(player2, 0), Ok(()));
		assert_eq!(MatchMaker::do_queue_size(0), 2);
		assert_eq!(MatchMaker::do_try_match(), [1, 2]);
		assert_eq!(MatchMaker::do_queue_size(0), 0);
		assert!(MatchMaker::do_try_match().is_empty());

		assert_eq!(MatchMaker::do_add_queue(player1, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(player2, 0), Ok(()));
		assert_eq!(MatchMaker::do_queue_size(0), 2);
		MatchMaker::do_empty_queue(0);
		assert!(MatchMaker::do_try_match().is_empty());
		assert_eq!(MatchMaker::do_queue_size(0), 0);
	});
}

#[test]
fn test_brackets_count() {
	new_test_ext().execute_with(|| {
		assert_eq!(MatchMaker::brackets_count(), 3);
	});
}

#[test]
fn test_brackets() {
	new_test_ext().execute_with(|| {
		let player1 = 1; // bracket: 0
		let player2 = 2; // bracket: 0
		let player3 = 3; // bracket: 0
		let player4 = 4; // bracket: 1
		let player5 = 5; // bracket: 1
		let player6 = 6; // bracket: 2

		assert_eq!(MatchMaker::do_queue_size(0), 0);
		assert_eq!(MatchMaker::do_all_queue_size(), 0);
		assert_eq!(MatchMaker::do_add_queue(player1, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(player2, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(player3, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(player4, 1), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(player5, 1), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(player6, 2), Ok(()));
		assert_eq!(MatchMaker::do_queue_size(0), 3);
		assert_eq!(MatchMaker::do_queue_size(1), 2);
		assert_eq!(MatchMaker::do_queue_size(2), 1);
		assert_eq!(MatchMaker::do_all_queue_size(), 6);
		assert_eq!(MatchMaker::do_try_match(), [1, 2]);
		assert_eq!(MatchMaker::do_try_match(), [3, 4]);
		assert_eq!(MatchMaker::do_add_queue(player1, 0), Ok(()));
		assert_eq!(MatchMaker::do_try_match(), [1, 5]);
		assert!(MatchMaker::do_try_match().is_empty());
		assert_eq!(MatchMaker::do_add_queue(player5, 1), Ok(()));
		assert_eq!(MatchMaker::do_try_match(), [5, 6]);
	});
}

#[test]
fn test_match_all_random() {
	new_test_ext().execute_with(|| {
		let empty: Vec<Vec<u64>> = vec![];

		assert_eq!(MatchMaker::do_queue_size(0), 0);
		assert_eq!(MatchMaker::do_all_queue_size(), 0);
		assert_eq!(MatchMaker::do_add_queue(1, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(2, 0), Ok(()));
		assert_eq!(MatchMaker::do_try_match_all_random(0, &Default::default()), [[1, 2]]);
		assert_eq!(MatchMaker::do_try_match_all_random(0, &Default::default()), empty);

		assert_eq!(MatchMaker::do_add_queue(1, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(2, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(3, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(4, 0), Ok(()));
		assert_eq!(MatchMaker::do_try_match_all_random(0, &Default::default()), [[1, 2], [3, 4]]);
		assert_eq!(MatchMaker::do_queue_size(0), 0);

		let seed = Blake2_128::hash(&2u8.encode());

		assert_eq!(MatchMaker::do_add_queue(1, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(2, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(3, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(4, 0), Ok(()));
		assert_eq!(MatchMaker::do_try_match_all_random(0, &seed), [[1, 4], [2, 3]]); // Might fail if the underlaying logic changes
		assert_eq!(MatchMaker::do_queue_size(0), 0);

		let seed = Blake2_128::hash(&1u8.encode());

		assert_eq!(MatchMaker::do_add_queue(1, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(2, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(3, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(4, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(5, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(6, 0), Ok(()));
		assert_eq!(MatchMaker::do_add_queue(7, 0), Ok(()));
		assert_eq!(MatchMaker::do_try_match_all_random(0, &seed), [[5, 6], [3, 4], [1, 2]]); // Might fail if the underlaying logic changes
		assert_eq!(MatchMaker::do_queue_size(0), 1);
	});
}
