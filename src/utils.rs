pub fn pluralize(n: usize) -> &'static str {
  if n == 1 {
    return "";
  }
  return "s";
}
