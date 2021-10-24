use std::process::Command;

/// This macro takes a unique identifier and creates a test with that name.
///
/// The test runs the binary crate using the input CSV file with that name
/// from the `tests/input` folder. It then asserts that the actual output
/// matches the expected output using the output CSV file with that same name
/// from the `tests/output` folder. All files must have the `.csv` extension.
macro_rules! test_csv {
    ($test_name:ident) => {
        #[test]
        fn $test_name() {
            let input_path = concat!("tests/input/", stringify!($test_name), ".csv");
            let output_path = concat!("tests/output/", stringify!($test_name), ".csv");

            let output = Command::new("cargo")
                .args(["run", "--release", "--", input_path])
                .output()
                .unwrap();

            let actual = String::from_utf8(output.stdout).unwrap();
            let expected = std::fs::read_to_string(output_path).unwrap();

            assert_eq!(actual, expected);
        }
    };
}

test_csv!(basic);
test_csv!(chargeback_deposit);
test_csv!(chargeback_negative);
test_csv!(chargeback_withdrawal);
test_csv!(dispute_deposit);
test_csv!(dispute_negative);
test_csv!(dispute_withdrawal);
test_csv!(resolve_deposit);
test_csv!(resolve_withdrawal);
