from unittest.mock import Mock

crs = Mock()
foo = Mock()
x = 12
y = 13
schema = "dbo"

print("don't want to match this")
foo.bar("don't want to match this either")


q = """
INSERT INTO bar VALUES (1)
"""

"""
INSERT INTO bar VALUES (1)
"""



crs.execute(q)
crs.execute('SELECT * FROM foo')
crs.execute(f'SELECT * FROM foo WHERE x = {x}')
crs.execute("SELECT * FROM foo")
crs.execute(f"SELECT * FROM foo WHERE x = {x}")


crs.execute(f"""SELECT * FROM foo""")

# SELECT
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
crs.execute(f"""
    SELECT *
    FROM {schema}.foo
    WHERE x = ?;


    DECLARE @my_table (
        foo INT,
        bar VARCHAR(20)
    );
    SELECT
        42,
        'hello'
    INTO @my_table;
""", x)

crs.executemany("SELECT * FROM foo")
crs.executemany(f"SELECT * FROM foo where x = {x}")


crs.execute('SELECT [update] FROM foo')
