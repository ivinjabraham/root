use root::db::leaderboard::Leaderboard;
use root::db::member::Member;
use root::leaderboard::fetch_stats::{fetch_codeforces_stats, fetch_leetcode_stats};
use root::leaderboard::update_leaderboard::update_leaderboard;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use std::sync::Arc;

pub fn get_database_url() -> String {
    match env::var("TEST_DATABASE_URL") {
        Ok(db_url) => db_url,
        Err(_) => "postgres://localhost:5432/default_db".to_string(),
    }
}

// Helper function to create a test database connection
async fn setup_test_db() -> PgPool {
    let database_url = get_database_url();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create test database pool");

    // Create tables if they do not already exist
    let queries = vec![
        r#"
        CREATE TABLE IF NOT EXISTS member (
            id SERIAL PRIMARY KEY,
            rollno VARCHAR(255) NOT NULL,
            name VARCHAR(255) NOT NULL,
            hostel VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL UNIQUE,
            sex VARCHAR(10) NOT NULL,
            year INT NOT NULL,
            macaddress VARCHAR(17) NOT NULL,
            discord_id VARCHAR(255),
            group_id INT NOT NULL
        )"#,
        r#"
        CREATE TABLE IF NOT EXISTS leaderboard (
            id SERIAL PRIMARY KEY,
            member_id INT UNIQUE NOT NULL,
            leetcode_score INT,
            codeforces_score INT,
            unified_score INT NOT NULL,
            last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (member_id) REFERENCES member(id)
        )"#,
        r#"
        CREATE TABLE IF NOT EXISTS leetcode_stats (
            id SERIAL PRIMARY KEY,
            member_id INT UNIQUE NOT NULL,
            leetcode_username VARCHAR(255) NOT NULL,
            problems_solved INT NOT NULL,
            easy_solved INT NOT NULL,
            medium_solved INT NOT NULL,
            hard_solved INT NOT NULL,
            contests_participated INT NOT NULL,
            best_rank INT NOT NULL,
            total_contests INT NOT NULL,
            FOREIGN KEY (member_id) REFERENCES member(id)
        )"#,
        r#"
        CREATE TABLE IF NOT EXISTS codeforces_stats (
            id SERIAL PRIMARY KEY,
            member_id INT UNIQUE NOT NULL,
            codeforces_handle VARCHAR(255) NOT NULL,
            codeforces_rating INT NOT NULL,
            max_rating INT NOT NULL,
            contests_participated INT NOT NULL,
            FOREIGN KEY (member_id) REFERENCES member(id)
        )"#,
    ];

    for query in queries {
        sqlx::query(query)
            .execute(&pool)
            .await
            .expect("Failed to execute query");
    }
    pool
}

// Helper function to clean up test data

async fn cleanup_test_data(pool: &PgPool) {
    print!("called");
    let cleanup_query = r#"
        DO $$
        DECLARE
            seq RECORD;
        BEGIN
            -- Droppign all the tables for cleanup purpose
            BEGIN
                TRUNCATE TABLE leaderboard, leetcode_stats, codeforces_stats, member RESTART IDENTITY CASCADE;
            EXCEPTION
                WHEN undefined_table THEN
                    -- Ignore errors if tables don't exist
                    RAISE NOTICE 'Tables do not exist, skipping TRUNCATE.';
            END;

            -- Postgres stores the sequences of unique id outside of respective tables, so need to delete those too. 
            FOR seq IN
                SELECT c.relname
                FROM pg_class c
                JOIN pg_namespace n ON n.oid = c.relnamespace
                WHERE c.relkind = 'S' AND n.nspname = 'public'
            LOOP
                BEGIN
                    EXECUTE 'ALTER SEQUENCE ' || seq.relname || ' RESTART WITH 1';
                EXCEPTION
                    WHEN undefined_object THEN
                        -- Ignore errors if sequences don't exist
                        RAISE NOTICE 'Sequence % does not exist, skipping.', seq.relname;
                END;
            END LOOP;
        END $$;
    "#;

    sqlx::query(cleanup_query)
        .execute(pool)
        .await
        .expect("Failed to clean up and reset database state");
}

#[tokio::test]
// Additional helper test to verify database connections and basic operations
async fn test_database_connection() {
    let database_url = get_database_url();
    println!("Database URL: {}", database_url);
    assert!(!database_url.is_empty(), "Database URL should not be empty");
}

//test
#[tokio::test]
async fn test_insert_members_and_update_stats() {
    let pool = setup_test_db().await;

    // Define test members
    let members = vec![
        (
            "B21CS1234",
            "John Doe",
            "Hostel A",
            "john.doe@example.com",
            "Male",
            2021,
            "00:11:22:33:44:55",
            Some("john_discord"),
            1,
            "swayam-agrahari",
            "tourist",
        ),
        (
            "B21CS5678",
            "Jane Smith",
            "Hostel B",
            "jane.smith@example.com",
            "Female",
            2021,
            "66:77:88:99:AA:BB",
            Some("jane_discord"),
            2,
            "rihaan1810",
            "tourist",
        ),
    ];

    let mut inserted_members = Vec::new();

    // Insert members and store their IDs
    for member in &members {
        // Insert Member
        let member_result = sqlx::query_as::<_, Member>(
            "INSERT INTO member (rollno, name, hostel, email, sex, year, macaddress, discord_id, group_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 RETURNING *",
        )
        .bind(&member.0)
        .bind(&member.1)
        .bind(&member.2)
        .bind(&member.3)
        .bind(&member.4)
        .bind(member.5)
        .bind(&member.6)
        .bind(&member.7)
        .bind(&member.8)
        .fetch_one(&pool)
        .await
        .expect("Failed to insert member");

        // Insert LeetCode stats
        let _leetcode_result = sqlx::query(
                "INSERT INTO leetcode_stats (member_id, leetcode_username,problems_solved,easy_solved,medium_solved,hard_solved,contests_participated,best_rank,total_contests)
                 VALUES ($1, $2, 0,0,0,0,0,0,0)",
            )
            .bind(member_result.id)
            .bind(&member.9)
            .execute(&pool)
            .await
            .expect("Failed to insert LeetCode stats");

        // Insert Codeforces stats
        let _codeforces_result = sqlx::query(
                "INSERT INTO codeforces_stats (member_id, codeforces_handle,codeforces_rating,max_rating,contests_participated)
                 VALUES ($1, $2, 0,0,0)",
            )
            .bind(member_result.id)
            .bind(&member.10)
            .execute(&pool)
            .await
            .expect("Failed to insert Codeforces stats");

        inserted_members.push(member_result.id);
    }

    // Test LeetCode stats fetching
    for (member_id, leetcode_username) in inserted_members.iter().zip(members.iter().map(|m| m.9)) {
        match fetch_leetcode_stats(Arc::new(pool.clone()), *member_id, leetcode_username).await {
            Ok(_) => println!(
                "Successfully fetched LeetCode stats for member ID: {}",
                member_id
            ),
            Err(e) => {
                println!("Error fetching LeetCode stats: {:?}", e);
                // Uncomment to fail test on fetch error
                // panic!("Failed to fetch LeetCode stats")
            }
        }
    }

    // Test Codeforces stats fetching
    for (member_id, codeforces_handle) in inserted_members.iter().zip(members.iter().map(|m| m.9)) {
        match fetch_codeforces_stats(Arc::new(pool.clone()), *member_id, codeforces_handle).await {
            Ok(_) => println!(
                "Successfully fetched Codeforces stats for member ID: {}",
                member_id
            ),
            Err(e) => {
                println!("Error fetching Codeforces stats: {:?}", e);
            }
        }
    }

    // Test leaderboard update
    match update_leaderboard(Arc::new(pool.clone())).await {
        Ok(_) => println!("Successfully updated leaderboard"),
        Err(e) => panic!("Failed to update leaderboard: {:?}", e),
    }

    // Verify leaderboard entries
    let leaderboard_entries = sqlx::query_as::<_, Leaderboard>("SELECT * FROM leaderboard")
        .fetch_all(&pool)
        .await
        .unwrap();

    assert_eq!(
        leaderboard_entries.len(),
        2,
        "Should have 2 leaderboard entries"
    );

    // Assertions about leaderboard scores
    for entry in leaderboard_entries {
        assert!(entry.unified_score > 0, "Unified score should be positive");
        assert!(
            entry.leetcode_score.is_some(),
            "LeetCode score should be set"
        );
        assert!(
            entry.codeforces_score.is_some(),
            "Codeforces score should be set"
        );
    }

    cleanup_test_data(&pool).await;
}
