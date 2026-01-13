/*
 * pgtimewarp.c
 *
 * PostgreSQL extension for time travel queries
 * 
 * This is a minimal C extension that loads the SQL functions.
 * The actual time travel logic is implemented in SQL (pgtimewarp--0.1.sql).
 */

#include "postgres.h"
#include "fmgr.h"

PG_MODULE_MAGIC;

/*
 * Extension initialization
 * 
 * This function is called when the extension is loaded.
 * For this MVP, all functionality is in SQL functions,
 * so we just need to ensure the module is properly initialized.
 */
void
_PG_init(void)
{
    /* Extension loaded successfully */
    elog(DEBUG1, "pgtimewarp extension loaded");
}

/*
 * Extension cleanup
 * 
 * Called when the extension is unloaded.
 */
void
_PG_fini(void)
{
    /* Extension unloaded */
    elog(DEBUG1, "pgtimewarp extension unloaded");
}
