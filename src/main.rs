use sqlx::postgres::PgPoolOptions;
use anyhow::Result;


#[actix_web::main]
async fn main() -> Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://admin:password@localhost:55432/test").await?;

    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool).await?;
    assert_eq!(row.0, 150);

    let create_table = r#"
    --- Make a dummy table
    CREATE TABLE IF NOT EXISTS helloworld (
            id      integer,
            set     hll
    );
    "#;

    let insert = r#"
    INSERT INTO helloworld(id, set) VALUES (1, hll_empty())
    "#;

    let add_sql = r#"
    UPDATE helloworld SET set = hll_add(set, hll_hash_integer(12345)) WHERE id = 1;
    "#;

    sqlx::query(create_table).execute(&pool).await?;
    sqlx::query(insert).execute(&pool).await?;
    sqlx::query(add_sql).execute(&pool).await?;

    let row: (f64,) = sqlx::query_as(
        "SELECT hll_cardinality(set) FROM helloworld WHERE id = 1"
    ).fetch_one(&pool).await?;
    println!("{}", row.0);

    Ok(())
}
