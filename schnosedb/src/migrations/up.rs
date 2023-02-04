use {
	super::schemas,
	color_eyre::Result as Eyre,
	log::info,
	sqlx::{MySql, Pool},
};

pub async fn up(pool: &Pool<MySql>) -> Eyre<()> {
	let query_string = schemas::players::up();
	info!("creating table `players`...");
	sqlx::query(query_string)
		.execute(pool)
		.await?;
	info!("successfully created table `players`.");

	let query_string = schemas::modes::up();
	info!("creating table `modes`...");
	sqlx::query(query_string)
		.execute(pool)
		.await?;
	info!("successfully created table `modes`.");

	let query_string = schemas::servers::up();
	info!("creating table `servers`...");
	sqlx::query(query_string)
		.execute(pool)
		.await?;
	info!("successfully created table `servers`.");

	let query_string = schemas::maps::up();
	info!("creating table `maps`...");
	sqlx::query(query_string)
		.execute(pool)
		.await?;
	info!("successfully created table `maps`.");

	let query_string = schemas::courses::up();
	info!("creating table `courses`...");
	sqlx::query(query_string)
		.execute(pool)
		.await?;
	info!("successfully created table `courses`.");

	let query_string = schemas::records::up();
	info!("creating table `records`...");
	sqlx::query(query_string)
		.execute(pool)
		.await?;
	info!("successfully created table `records`.");

	Ok(())
}
