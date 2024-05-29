((comment) @content
  (#set! language "comment"))

((line_continuation_comment) @content
  (#set! language "comment"))

((shell_fragment) @content
  (#set! language "bash")
  (#set! include-children))
