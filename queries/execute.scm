(call
  (attribute
    object: (identifier) @object (#eq? @object "crs")
    attribute: (identifier) @attribute (#any-of? @attribute "execute" "executemany"))
  (argument_list
    (string
      (string_start) @ss
      ; (string_content) @sql this splits the string
      (string_end) @se)))
