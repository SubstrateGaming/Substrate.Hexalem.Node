use crate::{mock::*, Event, RatingStorage};

#[test]
fn test_equal_elo() {
	new_test_ext().execute_with(|| {
		assert_eq!(EloModule::get_rating(&1), 1000);
		assert_eq!(EloModule::get_rating(&2), 1000);

		EloModule::do_update_rating(&1, &2);

		assert_eq!(EloModule::get_rating(&1), 1016);
		assert_eq!(EloModule::get_rating(&2), 984);

		EloModule::do_update_rating(&1, &2);

		assert_eq!(EloModule::get_rating(&1), 1032);
		assert_eq!(EloModule::get_rating(&2), 968);

		EloModule::do_update_rating(&1, &2);

		assert_eq!(EloModule::get_rating(&1), 1047);
		assert_eq!(EloModule::get_rating(&2), 953);

		// profesional level matches
		RatingStorage::<TestRuntime>::set(&3, 2000);
		RatingStorage::<TestRuntime>::set(&4, 2000);

		EloModule::do_update_rating(&3, &4);

		assert_eq!(EloModule::get_rating(&3), 2016);
		assert_eq!(EloModule::get_rating(&4), 1984); // Now I am reading 1984 book. No, I do not read, I only listen to audiobooks.

		// noob level matches
		RatingStorage::<TestRuntime>::set(&5, 100);
		RatingStorage::<TestRuntime>::set(&6, 100);

		EloModule::do_update_rating(&5, &6);

		assert_eq!(EloModule::get_rating(&5), 116);
		assert_eq!(EloModule::get_rating(&6), 84);
	});
}

#[test]
fn test_equal_elos() {
	new_test_ext().execute_with(|| {
		assert_eq!(EloModule::get_rating(&1), 1000);
		assert_eq!(EloModule::get_rating(&2), 1000);
		assert_eq!(EloModule::get_rating(&3), 1000);
		assert_eq!(EloModule::get_rating(&4), 1000);

		EloModule::do_update_ratings(&1, &vec![1, 2, 3, 4].try_into().unwrap());

		assert_eq!(EloModule::get_rating(&1), 1048);
		assert_eq!(EloModule::get_rating(&2), 984);
		assert_eq!(EloModule::get_rating(&3), 984);
		assert_eq!(EloModule::get_rating(&4), 984);
	});
}

#[test]
fn test_differences() {
	new_test_ext().execute_with(|| {
		RatingStorage::<TestRuntime>::set(2, 1200);

		EloModule::do_update_rating(&1, &2);

		assert_eq!(EloModule::get_rating(1), 1020);
		assert_eq!(EloModule::get_rating(2), 1180);
	});
}

#[test]
fn test_extreme_differences() {
	new_test_ext().execute_with(|| {
		RatingStorage::<TestRuntime>::set(&1, 10000);
		RatingStorage::<TestRuntime>::set(&2, 100);

		EloModule::do_update_rating(&1, &2);

		assert_eq!(EloModule::get_rating(&1), 10001);
		assert_eq!(EloModule::get_rating(&2), 99);

		EloModule::do_update_rating(&2, &1);

		assert_eq!(EloModule::get_rating(&1), 9969);
		assert_eq!(EloModule::get_rating(&2), 131);
	});
}

#[test]
fn test_lower_bound() {
	new_test_ext().execute_with(|| {
		RatingStorage::<TestRuntime>::set(&1, 1);
		RatingStorage::<TestRuntime>::set(&2, 1);

		EloModule::do_update_rating(&1, &2);

		assert_eq!(EloModule::get_rating(&1), 17);
		assert_eq!(EloModule::get_rating(&2), 0);

		RatingStorage::<TestRuntime>::set(&1, 1);

		EloModule::do_update_rating(&1, &2);

		assert_eq!(EloModule::get_rating(&1), 17);
		assert_eq!(EloModule::get_rating(&2), 0);
	});
}

#[test]
fn test_lower_bounds() {
	new_test_ext().execute_with(|| {
		RatingStorage::<TestRuntime>::set(&1, 1);
		RatingStorage::<TestRuntime>::set(&2, 1);
		RatingStorage::<TestRuntime>::set(&3, 1);

		EloModule::do_update_ratings(&1, &vec![1, 2, 3].try_into().unwrap());

		assert_eq!(EloModule::get_rating(&1), 33);
		assert_eq!(EloModule::get_rating(&2), 0);
		assert_eq!(EloModule::get_rating(&3), 0);
	});
}

#[test]
fn test_upper_bound() {
	new_test_ext().execute_with(|| {
		// u16 max value is 65535
		RatingStorage::<TestRuntime>::set(&1, 65535);
		RatingStorage::<TestRuntime>::set(&2, 65535);

		EloModule::do_update_rating(&1, &2);

		assert_eq!(EloModule::get_rating(&1), 65535);
		assert_eq!(EloModule::get_rating(&2), 65519);
	});
}

#[test]
fn test_update_rating_events() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		RatingStorage::<TestRuntime>::set(1, 1300);

		EloModule::do_update_rating(&2, &1);

		System::assert_has_event(
			Event::RatingGained { player: 2, new_rating: 1024, rating_gained: 24 }.into(),
		);

		System::assert_has_event(
			Event::RatingLost { player: 1, new_rating: 1276, rating_lost: 24 }.into(),
		);
	});
}

#[test]
fn test_update_ratings_events() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		RatingStorage::<TestRuntime>::set(1, 1300);

		RatingStorage::<TestRuntime>::set(3, 800);

		EloModule::do_update_ratings(&2, &vec![1, 2, 3, 4].try_into().unwrap());

		System::assert_has_event(
			Event::RatingGained { player: 2, new_rating: 1052, rating_gained: 52 }.into(),
		);

		System::assert_has_event(
			Event::RatingLost { player: 1, new_rating: 1276, rating_lost: 24 }.into(),
		);

		System::assert_has_event(
			Event::RatingLost { player: 3, new_rating: 788, rating_lost: 12 }.into(),
		);

		System::assert_has_event(
			Event::RatingLost { player: 4, new_rating: 984, rating_lost: 16 }.into(),
		);
	});
}
