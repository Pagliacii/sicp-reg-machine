pub mod register;

#[cfg(test)]
mod tests {
    use super::register::Register;

    #[test]
    fn test_get_register_contents() {
        let reg: Register = Register::new();
        let right: String = String::from("*unassigned*");
        assert!(reg.get().is::<String>());
        if let Some(left) = reg.get().downcast_ref::<String>() {
            assert_eq!(left, &right);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_set_register_contents() {
        let mut reg: Register = Register::new();
        let right: i32 = 12345678;
        reg.set(&right);
        if let Some(left) = reg.get().downcast_ref::<i32>() {
            assert_eq!(left, &right);
        } else {
            assert!(false);
        }
    }
}
