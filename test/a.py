
from unittest.mock import Mock

crs = Mock()

crs.execute("SELECT * FROM foo")

crs.execute("SELECT * FROM foo")
crs.execute("SELECT * FROM foo")
crs.execute("SELECT * FROM foo")
crs.execute("SELECT * FROM foo")
