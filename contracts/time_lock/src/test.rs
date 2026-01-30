#![cfg(test)]

mod tests {
    use crate::*;

    #[test]
    fn test_calculate_rate_difference() {
        let diff = slippage::calculate_rate_difference(1000000, 1010000);
        assert_eq!(diff, 100);

        let diff = slippage::calculate_rate_difference(1000000, 1050000);
        assert_eq!(diff, 500);

        let diff = slippage::calculate_rate_difference(1000000, 990000);
        assert_eq!(diff, -100);
    }

    #[test]
    fn test_calculate_expected_output() {
        let output = slippage::calculate_expected_output(1000000, 1000, 200);
        assert_eq!(output, 980);
    }

    #[test]
    fn test_validate_rate_bounds() {
        assert!(oracle::validate_rate_bounds(1000000).is_ok());
        assert!(oracle::validate_rate_bounds(0).is_err());
        assert!(oracle::validate_rate_bounds(-1000000).is_err());
    }

    #[test]
    fn test_validate_slippage_bounds() {
        assert!(slippage::validate_slippage_bounds(200).is_ok());
        assert!(slippage::validate_slippage_bounds(10000).is_ok());
        assert!(slippage::validate_slippage_bounds(10001).is_err());
    }

    #[test]
    fn test_rate_difference_calculations() {
        // Test various rate differences
        assert_eq!(slippage::calculate_rate_difference(1000000, 1000000), 0);
        assert_eq!(slippage::calculate_rate_difference(1000000, 1100000), 1000); // 10%
        assert_eq!(slippage::calculate_rate_difference(2000000, 2200000), 1000); // 10%
        assert_eq!(slippage::calculate_rate_difference(500000, 450000), -1000); // -10%
    }

    #[test]
    fn test_expected_output_calculations() {
        // Base case: 1000 units * 1.0 rate = 1000 with 2% slippage = 980
        assert_eq!(slippage::calculate_expected_output(1000000, 1000, 200), 980);

        // No slippage
        assert_eq!(slippage::calculate_expected_output(1000000, 1000, 0), 1000);

        // Max slippage 10%
        let output = slippage::calculate_expected_output(1000000, 1000, 1000);
        assert_eq!(output, 900); // 1000 - (1000 * 1000 / 10000) = 900

        // Different exchange rate
        assert_eq!(slippage::calculate_expected_output(2000000, 500, 200), 980);
        // Base: (500 * 2000000) / 1000000 = 1000
        // Slippage: (1000 * 200) / 10000 = 20
        // Result: 1000 - 20 = 980
    }
}
