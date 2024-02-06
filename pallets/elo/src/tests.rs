use crate::{mock::*, RatingStorage};

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
