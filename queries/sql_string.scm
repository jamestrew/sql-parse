;; this might be slow
;; the `contains?` match is case sensitive
(string
  (string_start) @ss
  (string_content) @str (#contains? @str "SELECT" "FROM" "WHERE" "UPDATE" "INSERT")
  (string_end) @se)

