
from unittest.mock import Mock

crs = Mock()

crs.execute("SELECT * FROM foo")

print("hello")
crs.execute("SELECT * FROM foo")
