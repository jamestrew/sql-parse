(call
  (attribute
    object: (identifier) @object (#eq? @object "crs")
    attribute: (identifier) @attribute (#any-of? @attribute "execute" "executemany"))
  (argument_list
    (string
      (string_content) @sql))
)

