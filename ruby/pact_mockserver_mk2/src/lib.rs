#[macro_use]
extern crate helix;

ruby! {
  class TestConsole {
      def log(message: String) {
          println!("LOG: {:?}", message);
      }
  }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
