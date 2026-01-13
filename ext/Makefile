EXTENSION = pgtimewarp
DATA = sql/pgtimewarp--0.1.sql
MODULE_big = pgtimewarp
OBJS = src/pgtimewarp.o

PG_CONFIG = pg_config
PGXS := $(shell $(PG_CONFIG) --pgxs)
include $(PGXS)
