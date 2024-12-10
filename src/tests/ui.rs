use crate::asm::interpreter::{build_lps_list, find_string, build_bin_lps_list, find_bin};
use crate::ui::window::line_buffer;

#[test]
fn should_build_lps(){
   assert_eq!(build_lps_list("aabaa"),vec![0,1,0,1,2]);
   assert_eq!(build_lps_list("aaac"),vec![0,1,2,0]);
   assert_eq!(build_lps_list("aaacaaaa"),vec![0,1,2,0,1,2,3,3]);
}

#[test]
fn should_find_occurances(){
   assert_eq!(find_string("xxhelloxxgoodbye","hello"),vec![2]);
   assert_eq!(find_string("xxhelloxxgoodbyehello","hello"),vec![2,16]);
   assert_eq!(find_string("hellh:/elehello","hello"),vec![10]);
   assert_eq!(find_string("hello","goodbye"),vec![]);
}

#[test]
fn should_build_bin_lps(){
   assert_eq!(build_bin_lps_list(&vec![0]),vec![0]);
   assert_eq!(build_bin_lps_list(&vec![1,1,33,1,1]),vec![0,1,0,1,2]);
   assert_eq!(build_bin_lps_list(&vec![7,12,58,1]),vec![0,0,0,0]);
   assert_eq!(build_bin_lps_list(
         &vec![7,12,14,14,7,14,14,1]),
         vec! [0, 0, 0, 0,1, 0, 0,0]
   );
}

#[test]
fn should_find_bin_occurances(){
   assert_eq!(find_bin(&vec![0,10,9,4,5,2,10,9,11,200], &vec![10,9]),vec![1,6]);
   assert_eq!(find_bin(&vec![19,20,11,32,13,14], &vec![19,9]),vec![]);
   assert_eq!(find_bin(&vec![10,8,2], &vec![19]),vec![]);
}

#[test]
fn should_bound_line_buffers(){
   assert_eq!(
      line_buffer(
         &concat!(
            "\n",
            "\n",
            "hello\n",
            "goodbye"
         ).into(),
         0,
         2
      ),
      "\n\nhello"
   );

   assert_eq!(
      line_buffer(
         &concat!(
            "something\n",
            "irrelevant\n",
            "what i \n",
            "actually care \n",
            "about :)\n",
            "more nonsense"
         ).into(),
         2,
         4
      ),
      "what i \nactually care \nabout :)"
   );

   assert_eq!(
      line_buffer(
         &concat!(
            "\n",
            "\n",
            "and for my next trick\n",
            "a dance\n"
         ).into(),
         2,
         3
      ),
      "and for my next trick\na dance"
   );
}
