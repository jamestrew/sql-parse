from unittest.mock import Mock

crs = Mock()
foo = Mock()
x = 12
y = 13

print("don't want to match this")
foo.bar("don't want to match this either")



crs.execute('SELECT * FROM foo')
crs.execute(f'SELECT * FROM foo WHERE x = {x}')
crs.execute("SELECT * FROM foo")
crs.execute(f"SELECT * FROM foo WHERE x = {x}")


crs.execute(f"""SELECT * FROM foo""")


crs.execute(f"""
    SELECT * FROM foo
""")
crs.execute(f'''
    SELECT * FROM foo
''')
crs.execute(f"""
    SELECT * FROM foo where x = {x} AND y = {y}
""")
crs.execute(f"""
    SELECT * FROM foo WHERE x = ?
""", x)

crs.execute("""
    SELECT * FROM foo
""")
crs.execute("""
    SELECT * FROM foo WHERE x = ?
""", x)

crs.executemany("SELECT * FROM foo")
crs.executemany(f"SELECT * FROM foo where x = {x}")

"""

(call
  (attribute
    object: (identifier) @object (#eq? @object "crs")
    attribute: (identifier) @attribute (#eq? @attribute "execute"))

  (string
    (string_content) @sql_string)
)

(string_content) @sql_string
"""
