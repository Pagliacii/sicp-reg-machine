/// An alternative version of the `Fn` trait
/// which encodes the type of the arguments
/// in a single type - a tuple.
pub trait Function<Args = ()>: Send + Sync + 'static {
    type Result;

    fn invoke(&self, args: Args) -> Self::Result;
}

macro_rules! func_impl {
    ( $( $name:ident )* ) => {
        impl<Func, Res, $($name),*> Function<($($name,)*)> for Func
        where
            Func: Fn($($name),*) -> Res + Send + Sync + 'static
        {
            type Result = Res;

            fn invoke(&self, args: ($($name,)*)) -> Self::Result {
                #[allow(non_snake_case)]
                let ($($name,)*) = args;
                (self)($($name,)*)
            }
        }
    };
}

func_impl! {}
func_impl! { A }
func_impl! { A B }
func_impl! { A B C }
func_impl! { A B C D }
func_impl! { A B C D E }
func_impl! { A B C D E F }
func_impl! { A B C D E F G }
func_impl! { A B C D E F G H }
func_impl! { A B C D E F G H I }
func_impl! { A B C D E F G H I J }
func_impl! { A B C D E F G H I J K }
func_impl! { A B C D E F G H I J K L }
func_impl! { A B C D E F G H I J K L M }
func_impl! { A B C D E F G H I J K L M N }
func_impl! { A B C D E F G H I J K L M N O }
func_impl! { A B C D E F G H I J K L M N O P }

#[cfg(test)]
mod function_mod_tests {
    use super::*;

    #[test]
    fn test_without_parameters() {
        fn zero() -> i32 {
            0
        }
        assert_eq!(zero.invoke(()), 0);
    }

    #[test]
    fn test_with_two_parameters() {
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
        assert_eq!(add.invoke((1, 2)), 3);
    }
}
