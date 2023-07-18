
from unittest.mock import Mock

crs = Mock()

crs.execute("""

    SELECT * FROM foo
SELECT * FROM foo
""")

print("hello")
crs.execute("SELECT * FROM foo")
