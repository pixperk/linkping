use crate::{errors::AppError, models::analytics::{AnalyticsRequest, ReferrerData, UserAgentData, ClickDistributionData, AnalyticsData, DateRange}};
use sqlx::{PgPool, query_as, Transaction, Postgres};


async fn build_filter_clause(slug: &str, params: &AnalyticsRequest) -> (String, Vec<String>, Option<DateRange>) {

    let mut date_filter = String::from("slug = $1");
    let mut query_params = vec![slug.to_string()];
    let mut arg_index = 2;

    let date_range = None;
    

    if let Some(start_date) = &params.start_date {
        date_filter.push_str(&format!(" AND timestamp::date >= ${}::date", arg_index));
        query_params.push(start_date.clone());
        arg_index += 1;
    }


    if let Some(end_date) = &params.end_date {
        date_filter.push_str(&format!(" AND timestamp::date <= ${}::date", arg_index));
        query_params.push(end_date.clone());
    }
    
    (date_filter, query_params, date_range)
}


async fn calculate_date_range(
    tx: &mut Transaction<'_, Postgres>,
    start_date: &str,
    end_date: &str
) -> Result<DateRange, AppError> {
    let days_query = "SELECT DATE_PART('day', $1::date - $2::date) + 1 as days";
    
    let (days,): (i64,) = query_as(days_query)
        .bind(end_date)
        .bind(start_date)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Days calculation: {}", e)))?;
        
    Ok(DateRange {
        start: start_date.to_string(),
        end: end_date.to_string(),
        days,
    })
}

async fn execute_count_query<T>(
    tx: &mut Transaction<'_, Postgres>,
    query: &str,
    params: &[&str],
    error_context: &str
) -> Result<T, AppError> 
where
    T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    let mut query_builder = query_as::<_, T>(query);
    

    for param in params {
        query_builder = query_builder.bind(*param);
    }
    
    query_builder
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("{}: {}", error_context, e)))
}


async fn execute_multi_query<T>(
    tx: &mut Transaction<'_, Postgres>,
    query: &str,
    params: &[&str],
    error_context: &str
) -> Result<Vec<T>, AppError> 
where
    T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    let mut query_builder = query_as::<_, T>(query);
    

    for param in params {
        query_builder = query_builder.bind(*param);
    }
    
    query_builder
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("{}: {}", error_context, e)))
}

pub async fn get_analytics_data(db: &PgPool, slug: String, params: &AnalyticsRequest) -> Result<AnalyticsData, AppError> {

    let mut tx = db.begin().await
        .map_err(|e| AppError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

 
    let (date_filter, params_vec, mut date_range) = build_filter_clause(&slug, params).await;
    

    let params_refs: Vec<&str> = params_vec.iter().map(|s| s.as_str()).collect();

    if let (Some(start), Some(end)) = (&params.start_date, &params.end_date) {
        date_range = Some(calculate_date_range(&mut tx, start, end).await?);
    }

    let total_clicks_query = format!("SELECT COUNT(*) FROM clicks WHERE {}", date_filter);
    let (total_clicks,): (i64,) = execute_count_query(
        &mut tx, 
        &total_clicks_query, 
        &params_refs, 
        "Total Clicks"
    ).await?;

    // Return early if no clicks found
    if total_clicks == 0 {
        return Err(AppError::NotFound(format!("No analytics found for slug '{}'", slug)));
    }

    // Get unique clicks (by IP)
    let unique_clicks_query = format!("SELECT COUNT(DISTINCT ip) FROM clicks WHERE {}", date_filter);
    let (unique_clicks,): (i64,) = execute_count_query(
        &mut tx, 
        &unique_clicks_query, 
        &params_refs, 
        "Unique Clicks"
    ).await?;

    // Get top referrers
    let referrer_limit = params.referer_quantity.unwrap_or(10);
    let top_referrers_query = format!(
        "SELECT COALESCE(referer, 'Unknown') AS referer, COUNT(*) as count
         FROM clicks WHERE {}
         GROUP BY referer
         ORDER BY count DESC
         LIMIT {}",
        date_filter, referrer_limit
    );
    
    let referrer_results: Vec<(String, i64)> = execute_multi_query(
        &mut tx, 
        &top_referrers_query, 
        &params_refs, 
        "Top Referrers"
    ).await?;
    
    let top_referrers = referrer_results
        .into_iter()
        .map(|(referer, count)| ReferrerData { referer, count })
        .collect();

    // Get top user agents
    let user_agent_limit = params.user_agent_quantity.unwrap_or(10);
    let top_user_agents_query = format!(
        "SELECT COALESCE(user_agent, 'Unknown') AS user_agent, COUNT(*) as count
         FROM clicks WHERE {}
         GROUP BY user_agent
         ORDER BY count DESC
         LIMIT {}",
        date_filter, user_agent_limit
    );
    
    let user_agent_results: Vec<(String, i64)> = execute_multi_query(
        &mut tx, 
        &top_user_agents_query, 
        &params_refs, 
        "Top User Agents"
    ).await?;
    
    let top_user_agents = user_agent_results
        .into_iter()
        .map(|(user_agent, count)| UserAgentData { user_agent, count })
        .collect();

    // Get click distribution by date
    let click_distribution_limit = params.click_distribution_quantity.unwrap_or(30);
    let click_distribution_query = format!(
        "SELECT to_char(timestamp, 'YYYY-MM-DD') AS date, COUNT(*) as count
         FROM clicks WHERE {}
         GROUP BY date
         ORDER BY date
         LIMIT {}",
        date_filter, click_distribution_limit
    );
    
    let distribution_results: Vec<(String, i64)> = execute_multi_query(
        &mut tx, 
        &click_distribution_query, 
        &params_refs, 
        "Click Distribution"
    ).await?;
    
    let click_distribution = distribution_results
        .into_iter()
        .map(|(date, count)| ClickDistributionData { date, count })
        .collect();

    // Commit the transaction
    tx.commit().await
        .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

    // Return the complete analytics data
    Ok(AnalyticsData {
        total_clicks,
        unique_clicks,
        top_referrers,
        top_user_agents,
        click_distribution,
        date_range,
    })
}
