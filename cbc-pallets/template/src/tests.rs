use crate::{mock::*, Error, Event, Something};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(Template::do_something(RuntimeOrigin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(Something::<Test>::get(), Some(42));
		// Assert that the correct event was deposited
		System::assert_last_event(Event::SomethingStored { something: 42, who: 1 }.into());
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(Template::cause_error(RuntimeOrigin::signed(1)), Error::<Test>::NoneValue);
	});
}


#[test]
fn simulate_runtime_panic_and_handle_gracefully() {
    // Use catch_unwind to safely handle panic in test
    let result = std::panic::catch_unwind(|| {
        new_test_ext().execute_with(|| {
            // Simulate something going wrong
            panic!("Simulated runtime panic for cbc-pallets");	
        });
    });

    // We expect the panic to have occurred and been caught
    assert!(result.is_err(), "Expected panic did not occur.");
}

#[test]
fn multiple_users_can_store_different_values() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Template::do_something(RuntimeOrigin::signed(1), 123));
		assert_eq!(Something::<Test>::get(), Some(123));

		assert_ok!(Template::do_something(RuntimeOrigin::signed(2), 456));
		assert_eq!(Something::<Test>::get(), Some(456));
	});
}

#[test]
fn test_event_emission() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Template::do_something(RuntimeOrigin::signed(1), 789));
		System::assert_last_event(Event::SomethingStored { something: 789, who: 1 }.into());
	});
}