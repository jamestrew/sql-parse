

crs.execute("SELECT * FROM foo")

crs.executemany("SELECT * FROM foo")
crs.executemany(f"SELECT * FROM foo where x = {x}")
crs.execute("SELECT * FROM foo")
crs.execute("SELECT * FROM foo")

crs.execute("""SELECT * FROM foo""")
crs.execute(f"""SELECT * FROM foo""")


crs.execute(f"""
    SELECT * FROM foo
""")
crs.execute(f"""
    SELECT * FROM foo where x = {x}
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
