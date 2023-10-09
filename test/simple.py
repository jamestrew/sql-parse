
from unittest.mock import Mock

crs = Mock()

crs.execute("""

    SELECT * FROM foo
SELECT * FROM foo
""")

print("hello")
f = "foo"
crs.execute("SELECT * FROM foo")
crs.execute("SELECT * FROM foo", "foo")
crs.execute(f"SELECT * FROM {f}")
crs.execute("SELECT * FROM {foo}".format(foo=f))
crs.execute("SELECT * FROM {foo}".format(foo=f), "foo")

