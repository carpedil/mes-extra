use nanoid::nanoid;

/// USER_TAB_COLUMNS_SQL
pub fn get_user_tab_columns_sql(wc: Option<String>) -> String {
    if let Some(where_clause) = wc {
        return format!(
            "
        WITH T_TABLE AS (
        SELECT
            A.TABLE_NAME,
            A.COLUMN_NAME,
            A.DATA_TYPE,
            A.DATA_LENGTH
        FROM
            USER_TAB_COLUMNS A
        )
        SELECT 
            A.TABLE_NAME,
            UTS.COMMENTS AS TAB_DESC,
            A.COLUMN_NAME,
            UCS.COMMENTS AS COL_DESC,
            A.DATA_TYPE,
            A.DATA_LENGTH
        FROM
            T_TABLE A
        LEFT JOIN USER_TAB_COMMENTS UTS ON
            A.TABLE_NAME = UTS.TABLE_NAME
        LEFT JOIN USER_COL_COMMENTS UCS ON
            A.TABLE_NAME = UCS.TABLE_NAME
            AND UCS.COLUMN_NAME = A.COLUMN_NAME
        WHERE
            A.TABLE_NAME NOT IN ({})
        ORDER BY
            A.TABLE_NAME",
            where_clause
        );
    } else {
        return format!(
            "
        WITH T_TABLE AS (
        SELECT
            A.TABLE_NAME,
            A.COLUMN_NAME,
            A.DATA_TYPE,
            A.DATA_LENGTH
        FROM
            USER_TAB_COLUMNS A
        )
        SELECT 
            A.TABLE_NAME,
            UTS.COMMENTS AS TAB_DESC,
            A.COLUMN_NAME,
            UCS.COMMENTS AS COL_DESC,
            A.DATA_TYPE,
            A.DATA_LENGTH
        FROM
            T_TABLE A
        LEFT JOIN USER_TAB_COMMENTS UTS ON
            A.TABLE_NAME = UTS.TABLE_NAME
        LEFT JOIN USER_COL_COMMENTS UCS ON
            A.TABLE_NAME = UCS.TABLE_NAME
            AND UCS.COLUMN_NAME = A.COLUMN_NAME
        ORDER BY
            A.TABLE_NAME"
        );
    }
}

pub fn remove_target_with_commas(mut source: Vec<&str>, target: &str) -> String {
    source.retain(|item| *item != target);
    source
        .iter()
        .map(|&s| s.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

pub fn gen_uid() -> String {
    nanoid!(6)
}
