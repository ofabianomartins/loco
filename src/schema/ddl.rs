#[macro_export]
macro_rules! create_table {
    ($manager:expr, $table_name:ident, $block:expr) => {
        async {
            let mut create_table = Table::create()
                .table(stringify!($table_name))
                .if_not_exists()
                .to_owned();
            $block(&mut create_table);
            $manager.create_table(create_table).await
        }
    };
}

#[macro_export]
macro_rules! drop_table {
    ($manager:expr, $table_name:ident) => {
        async {
            $manager
                .drop_table(Table::drop().table(stringify!($table_name)).to_owned())
                .await
        }
    };
}

#[macro_export]
macro_rules! add_column {
    ($manager:expr, $table_name:ident, $column:expr) => {
        async {
            $manager
                .alter_table(
                    Table::alter()
                        .table(stringify!($table_name))
                        .add_column($column)
                        .to_owned(),
                )
                .await
        }
    };
}

#[macro_export]
macro_rules! remove_column {
    ($manager:expr, $table_name:ident, $column_name:ident) => {
        async {
            $manager
                .alter_table(
                    Table::alter()
                        .table(stringify!($table_name))
                        .drop_column(stringify!($column_name))
                        .to_owned(),
                )
                .await
        }
    };
}

#[macro_export]
macro_rules! rename_column {
    ($manager:expr, $table_name:ident, $old_name:ident, $new_name:ident) => {
        async {
            $manager
                .alter_table(
                    Table::alter()
                        .table(stringify!($table_name))
                        .rename_column(stringify!($old_name), stringify!($new_name))
                        .to_owned(),
                )
                .await
        }
    };
}

#[macro_export]
macro_rules! add_index {
    ($manager:expr, $table_name:ident, $index:expr) => {
        async {
            $manager
                .create_index(
                    Index::create()
                        .table(stringify!($table_name))
                        .apply($index)
                        .to_owned(),
                )
                .await
        }
    };
}

#[macro_export]
macro_rules! remove_index {
    ($manager:expr, $table_name:ident, $index_name:ident) => {
        async {
            $manager
                .drop_index(
                    Index::drop()
                        .table(stringify!($table_name))
                        .name(stringify!($index_name))
                        .to_owned(),
                )
                .await
        }
    };
}

#[macro_export]
macro_rules! add_foreign_key {
    ($manager:expr, $from_table:ident, $to_table:ident, $block:expr) => {
        async {
            let mut fk = ForeignKey::create();
            fk.from(stringify!($from_table), "").to(stringify!($to_table), "");
            $block(&mut fk);
            $manager.create_foreign_key(fk.to_owned()).await
        }
    };
}

#[macro_export]
macro_rules! remove_foreign_key {
    ($manager:expr, $from_table:ident, $fk_name:ident) => {
        async {
            $manager
                .drop_foreign_key(
                    ForeignKey::drop()
                        .table(stringify!($from_table))
                        .name(stringify!($fk_name))
                        .to_owned(),
                )
                .await
        }
    };
}

#[macro_export]
macro_rules! rename_table {
    ($manager:expr, $old_name:ident, $new_name:ident) => {
        async {
            $manager
                .rename_table(
                    Table::rename()
                        .table(stringify!($old_name), stringify!($new_name))
                        .to_owned(),
                )
                .await
        }
    };
}
