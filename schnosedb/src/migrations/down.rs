use {
	super::schemas,
	color_eyre::Result as Eyre,
	log::{info, warn},
	sqlx::{MySql, Pool},
};

pub async fn down(pool: &Pool<MySql>) -> Eyre<()> {
	let query_string = schemas::records::down();
	warn!("dropping table `records`...");
	sqlx::query(query_string)
		.execute(pool)
		.await?;
	info!("successfully dropped table `records`.");

	let query_string = schemas::courses::down();
	warn!("dropping table `courses`...");
	sqlx::query(query_string)
		.execute(pool)
		.await?;
	info!("successfully dropped table `courses`.");

	let query_string = schemas::maps::down();
	warn!("dropping table `maps`...");
	sqlx::query(query_string)
		.execute(pool)
		.await?;
	info!("successfully dropped table `maps`.");

	let query_string = schemas::servers::down();
	warn!("dropping table `servers`...");
	sqlx::query(query_string)
		.execute(pool)
		.await?;
	info!("successfully dropped table `servers`.");

	let query_string = schemas::modes::down();
	warn!("dropping table `modes`...");
	sqlx::query(query_string)
		.execute(pool)
		.await?;
	info!("successfully dropped table `modes`.");

	let query_string = schemas::players::down();
	warn!("dropping table `players`...");
	sqlx::query(query_string)
		.execute(pool)
		.await?;
	info!("successfully dropped table `players`.");

	Ok(())
}
