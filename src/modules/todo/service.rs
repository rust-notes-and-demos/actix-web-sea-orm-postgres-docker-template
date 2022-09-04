use migration::DbErr;
use sea_orm::{query::*, ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, Set, ModelTrait};

use crate::shared::errors::CustomError;
use crate::shared::responses::CustomResponse;
use entity::todo;

pub async fn find_todos(
    conn: &DbConn,
    query_string: Option<String>,
    items_per_page: Option<usize>,
    page_num: Option<usize>,
) -> Result<Vec<todo::Model>, CustomError> {
    let mut stmt = todo::Entity::find();

    if let Some(query_string) = query_string {
        stmt = stmt.filter(todo::Column::Title.contains(query_string.as_str()));
    }

    let results = stmt
        .order_by_desc(todo::Column::UpdatedAt)
        .paginate(conn, items_per_page.unwrap_or(10))
        .fetch_page(page_num.unwrap_or(0))
        .await
        .map_err(|_| CustomError::ServerError)?;

    Ok(results)
}

pub async fn find_todo_by_id(conn: &DbConn, id: usize) -> Result<todo::Model, CustomError> {
    let result = todo::Entity::find_by_id(id as i32)
        .one(conn)
        .await
        .map_err(|_| CustomError::ServerError)?;

    if result.is_none() {
        return Err(CustomError::NotFound);
    }

    Ok(result.unwrap())
}

pub async fn insert_todo(
    conn: &DbConn,
    title: &str,
    description: &str,
    done: bool,
) -> Result<CustomResponse, CustomError> {
    let res = todo::Entity::insert(todo::ActiveModel {
        title: Set(title.to_string()),
        description: Set(description.to_string()),
        done: Set(done),
        ..Default::default()
    })
    .exec(conn)
    .await
    .map_err(|e| {
        match e {
            DbErr::Query(..) => CustomError::Conflict,
            _ => CustomError::ServerError,
        }
    })?;

    Ok(CustomResponse::Created {id: res.last_insert_id as usize})
}

pub async fn update_todo_by_id(
    conn: &DbConn,
    id: usize,
    title: Option<String>,
    description: Option<String>,
    done: Option<bool>,
) -> Result<CustomResponse, CustomError> {
    let todo = todo::Entity::find_by_id(id as i32)
        .one(conn)
        .await
        .map_err(|_| CustomError::ServerError)?;

    if todo.is_none() {
        return Err(CustomError::NotFound);
    }

    let mut todo: todo::ActiveModel = todo.unwrap().into();

    if let Some(title) = title {
        todo.title = Set(title);
    }

    if let Some(description) = description {
        todo.description = Set(description);
    }

    if let Some(done) = done {
        todo.done = Set(done);
    }

    todo.update(conn)
        .await
        .map_err(|e| {
            println!("Updated error: {:?}", e);
            CustomError::ServerError
        })?;

    Ok(CustomResponse::Updated { id })
}

pub async fn delete_todo_by_id(conn: &DbConn, id: usize) -> Result<CustomResponse, CustomError> {
    let found: todo::Model = find_todo_by_id(conn, id).await?;

    found.delete(conn).await.map_err(|_| CustomError::ServerError)?;

    Ok(CustomResponse::Deleted { id })
}

pub async fn bulk_delete_todos_by_ids(
    conn: &DbConn,
    ids: Vec<usize>,
) -> Result<CustomResponse, CustomError> {
    let txn = conn.begin().await.map_err(|e| {
        println!("Transaction error: {:?}", e);
        CustomError::ServerError
    })?;

    for id in ids.clone() {
        let found: todo::Model = find_todo_by_id(conn, id).await?;
        found.delete(&txn).await.map_err(|_| CustomError::ServerError)?;
    }

    txn.commit().await.map_err(|e| {
        println!("Transaction error: {:?}", e);
        CustomError::ServerError
    })?;

    Ok(CustomResponse::BulkDeleted { ids })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};
    use entity::todo;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult, Transaction};

    #[async_std::test]
    async fn test_find_todos() -> Result<(), CustomError> {
        let datetime = FixedOffset::east(0).ymd(2016, 11, 08).and_hms(0, 0, 0);
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![
                // First query result
                vec![
                    todo::Model {
                        id: 1,
                        title: "Todo 1".to_owned(),
                        description: "Todo 1 description".to_owned(),
                        done: false,
                        created_at: datetime,
                        updated_at: datetime,
                    },
                    todo::Model {
                        id: 2,
                        title: "Todo 2".to_owned(),
                        description: "Todo 2 description".to_owned(),
                        done: true,
                        created_at: datetime,
                        updated_at: datetime,
                    },
                ],
                // Second query result
                vec![
                    todo::Model {
                        id: 1,
                        title: "Apple pie".to_owned(),
                        description: "description".to_owned(),
                        done: false,
                        created_at: datetime,
                        updated_at: datetime,
                    },
                    todo::Model {
                        id: 3,
                        title: "Apple pizza".to_owned(),
                        description: "description".to_owned(),
                        done: false,
                        created_at: datetime,
                        updated_at: datetime,
                    },
                ],
                // Third query result
                vec![todo::Model {
                    id: 1,
                    title: "Apple pie".to_owned(),
                    description: "description".to_owned(),
                    done: false,
                    created_at: datetime,
                    updated_at: datetime,
                }],
            ])
            .into_connection();

        // testing find_todos with no query string
        assert_eq!(
            find_todos(&db, None, None, None).await?,
            vec![
                todo::Model {
                    id: 1,
                    title: "Todo 1".to_owned(),
                    description: "Todo 1 description".to_owned(),
                    done: false,
                    created_at: datetime,
                    updated_at: datetime,
                },
                todo::Model {
                    id: 2,
                    title: "Todo 2".to_owned(),
                    description: "Todo 2 description".to_owned(),
                    done: true,
                    created_at: datetime,
                    updated_at: datetime,
                }
            ]
        );

        // testing find_todos with query string
        assert_eq!(
            find_todos(&db, Some("Apple".to_owned()), None, None).await?,
            vec![
                todo::Model {
                    id: 1,
                    title: "Apple pie".to_owned(),
                    description: "description".to_owned(),
                    done: false,
                    created_at: datetime,
                    updated_at: datetime,
                },
                todo::Model {
                    id: 3,
                    title: "Apple pizza".to_owned(),
                    description: "description".to_owned(),
                    done: false,
                    created_at: datetime,
                    updated_at: datetime,
                }
            ]
        );

        // testing find_todos with query string and pagination
        assert_eq!(
            find_todos(&db, Some("Apple".to_owned()), Some(5), Some(1)).await?,
            vec![todo::Model {
                id: 1,
                title: "Apple pie".to_owned(),
                description: "description".to_owned(),
                done: false,
                created_at: datetime,
                updated_at: datetime,
            }]
        );

        // Checking transaction log
        assert_eq!(
            db.into_transaction_log(),
            vec![
                Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"SELECT "todo"."id", "todo"."title", "todo"."description", "todo"."done", "todo"."created_at", "todo"."updated_at" FROM "todo" ORDER BY "todo"."updated_at" DESC LIMIT $1 OFFSET $2"#,
                    vec![10u64.into(), 0u64.into()]
                ),
                Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"SELECT "todo"."id", "todo"."title", "todo"."description", "todo"."done", "todo"."created_at", "todo"."updated_at" FROM "todo" WHERE "todo"."title" LIKE $1 ORDER BY "todo"."updated_at" DESC LIMIT $2 OFFSET $3"#,
                    vec!["%Apple%".into(), 10u64.into(), 0u64.into()]
                ),
                Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"SELECT "todo"."id", "todo"."title", "todo"."description", "todo"."done", "todo"."created_at", "todo"."updated_at" FROM "todo" WHERE "todo"."title" LIKE $1 ORDER BY "todo"."updated_at" DESC LIMIT $2 OFFSET $3"#,
                    vec!["%Apple%".into(), 5u64.into(), 5u64.into()]
                ),
            ]
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_find_todo_by_id() -> Result<(), CustomError> {
        let datetime = FixedOffset::east(0).ymd(2016, 11, 08).and_hms(0, 0, 0);
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![
                // First query result
                vec![todo::Model {
                    id: 1,
                    title: "Todo 1".to_owned(),
                    description: "Todo 1 description".to_owned(),
                    done: false,
                    created_at: datetime,
                    updated_at: datetime,
                }],
                // Second query result
                vec![],
            ])
            .into_connection();

        // testing find_todo_by_id with existing id
        assert_eq!(
            find_todo_by_id(&db, 1).await?,
            todo::Model {
                id: 1,
                title: "Todo 1".to_owned(),
                description: "Todo 1 description".to_owned(),
                done: false,
                created_at: datetime,
                updated_at: datetime,
            }
        );

        // testing find_todo_by_id with non-existing id
        assert_eq!(
            find_todo_by_id(&db, 2).await.unwrap_err(),
            CustomError::NotFound
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_insert_todo() -> Result<(), CustomError> {
        let title = "Test Title";
        let description = "Test Description";
        let done = false;
        let datetime = FixedOffset::east(0).ymd(2016, 11, 08).and_hms(0, 0, 0);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![todo::Model {
                id: 15,
                title: title.to_owned(),
                description: description.to_owned(),
                done,
                created_at: datetime,
                updated_at: datetime,
            }]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 15,
                rows_affected: 1,
            }])
            .into_connection();

        insert_todo(&db, title, description, done).await?;

        assert_eq!(
            db.into_transaction_log(),
            vec![Transaction::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"INSERT INTO "todo" ("title", "description", "done") VALUES ($1, $2, $3) RETURNING "id""#,
                vec![title.into(), description.into(), done.into()]
            )]
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_update_todo_by_id() -> Result<(), CustomError> {
        let id = 1;
        let old_title = "Old Title";
        let title = "Test Title";
        let description = "Test Description";
        let done = false;
        let datetime = FixedOffset::east(0).ymd(2016, 11, 08).and_hms(0, 0, 0);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![
                // First query result
                vec![todo::Model {
                    id,
                    title: old_title.to_owned(),
                    description: description.to_owned(),
                    done,
                    created_at: datetime,
                    updated_at: datetime,
                }],
                // Second query result
                vec![todo::Model {
                    id,
                    title: old_title.to_owned(),
                    description: description.to_owned(),
                    done,
                    created_at: datetime,
                    updated_at: datetime,
                }],
                // Third query result
                vec![],
            ])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 15,
                rows_affected: 1,
            }])
            .into_connection();

        // testing update_todo_by_id with existing id
        update_todo_by_id(
            &db,
            id.try_into().unwrap(),
            Some(title.to_owned()),
            Some(description.to_owned()),
            Some(done),
        )
        .await?;

        // testing update_todo_by_id with non-existing id
        assert_eq!(
            update_todo_by_id(
                &db,
                2,
                Some(title.to_owned()),
                Some(description.to_owned()),
                Some(done)
            )
            .await
            .unwrap_err(),
            CustomError::NotFound
        );

        assert_eq!(
            db.into_transaction_log()[..2],
            vec![
                Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"SELECT "todo"."id", "todo"."title", "todo"."description", "todo"."done", "todo"."created_at", "todo"."updated_at" FROM "todo" WHERE "todo"."id" = $1 LIMIT $2"#,
                    vec![id.into(), 1u64.into()]
                ),
                Transaction::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    r#"UPDATE "todo" SET "title" = $1, "description" = $2, "done" = $3 WHERE "todo"."id" = $4 RETURNING "id", "title", "description", "done", "created_at", "updated_at""#,
                    vec![title.into(), description.into(), done.into(), id.into()]
                ),
            ]
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_delete_todo_by_id() -> Result<(), CustomError> {
        let id = 1;
        let datetime = FixedOffset::east(0).ymd(2016, 11, 08).and_hms(0, 0, 0);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![
                vec![todo::Model {
                    id,
                    title: "Todo 1".to_owned(),
                    description: "Todo 1 description".to_owned(),
                    done: false,
                    created_at: datetime,
                    updated_at: datetime,
                }],
                vec![],
            ])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            }])
            .append_exec_results(vec![])
            .into_connection();
        // testing delete_todo_by_id with existing id
        delete_todo_by_id(&db, id.try_into().unwrap()).await?;
        // testing delete_todo_by_id with non-existing id
        assert_eq!(
            delete_todo_by_id(&db, 2).await.unwrap_err(),
            CustomError::NotFound
        );
        assert_eq!(
            db.into_transaction_log()[1..2],
            vec![Transaction::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"DELETE FROM "todo" WHERE "todo"."id" = $1"#,
                vec![id.into()]
            )]
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_bulk_delete_todos() -> Result<(), CustomError> {
        let datetime = FixedOffset::east(0).ymd(2016, 11, 08).and_hms(0, 0, 0);
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![
                // First query result
                vec![todo::Model {
                    id: 1,
                    title: "test1".to_owned(),
                    description: "test1".to_owned(),
                    done: false,
                    created_at: datetime,
                    updated_at: datetime,
                }],
                // Second query result
                vec![todo::Model {
                    id: 2,
                    title: "test2".to_owned(),
                    description: "test2".to_owned(),
                    done: false,
                    created_at: datetime,
                    updated_at: datetime,
                }],
                // Third query result
                vec![todo::Model {
                    id: 3,
                    title: "test3".to_owned(),
                    description: "test3".to_owned(),
                    done: false,
                    created_at: datetime,
                    updated_at: datetime,
                }],
            ])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            }])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 2,
                rows_affected: 2,
            }])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 3,
                rows_affected: 3,
            }])
            .into_connection();
        bulk_delete_todos_by_ids(&db, vec![1, 2, 3]).await?;

        // skipped assertion as there is no non-trivial way to test a transaction with many statements

        Ok(())
    }
}
