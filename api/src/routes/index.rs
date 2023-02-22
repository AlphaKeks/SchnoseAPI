use axum::response::Html;

#[rustfmt::skip]
pub async fn get() -> Html<&'static str> {
	Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
	<meta charset="UTF-8">
	<meta http-equiv="X-UA-Compatible" content="IE=edge">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<title>SchnoseAPI (͡ ͡° ͜ つ ͡͡°)</title>
</head>

<style>

body {
	background-color: #1e1e2e;
	color: #cdd6f4;
}

</style>

<body>
	<h1>SchnoseAPI</h1>

	<h3><code>/api/modes/:ident</code></h3>
	</h4>Get 1 mode by an identifier. Identifier can be:</h4>
	<ul>
		<li>Name (e.g. <code>"kz_simple"</code>)</li>
		<li>ID (e.g. <code>201</code>)</li>
	</ul>

	<h3><code>/api/modes</code></h3>
	</h4>Get all modes.</h4>

	<h3><code>/api/players/:ident</code></h3>
	</h4>Get 1 player by an identifier. Identifier can be:</h4>
	<ul>
		<li>Name (e.g. <code>"AlphaKeks"</code> (THIS IS PRETTY SLOW))</li>
		<li>SteamID (e.g. <code>"STEAM_1:1:161178172"</code>)</li>
		<li>SteamID64 (e.g. <code>76561198282622073</code>)</li>
	</ul>

	<h3><code>/api/players</code></h3>
	</h4>Get up to 500 players. Parameters:</h4>
	<ul>
		<li><code>is_banned</code>: <code>Option&lt;bool&gt;</code></li>
		<li><code>limit</code>: <code>Option&lt;u32&gt;</code></li>
		<li><code>offset</code>: <code>Option&lt;i32&gt;</code></li>
	</ul>

	<h3><code>/api/servers/:ident</code></h3>
	</h4>Get a server by an identifier. Identifier can be:</h4>
	<ul>
		<li>Name (e.g. <code>"Hikari KZ"</code>)</li>
		<li>ID (e.g. <code>999</code>)</li>
	</ul>

	<h3><code>/api/servers</code></h3>
	</h4>Get servers. Parameters:</h4>
	<ul>
		<li><code>name</code>: <code>Option&lt;String&gt; (this can be an identifier just like above)</code></li>
		<li><code>owned_by</code>: <code>Option&lt;String&gt; (this is a player identifier just like above)</code></li>
		<li><code>approved_by</code>: <code>Option&lt;String&gt; (this is a player identifier just like above)</code></li>
		<li><code>limit</code>: <code>Option&lt;u32&gt;</code></li>
	</ul>

	<h3><code>/api/maps/:ident</code></h3>
	</h4>Get a map by an identifier. Identifier can be:</h4>
	<ul>
		<li>Name (e.g. <code>"lionharder"</code>)</li>
		<li>ID (e.g. <code>992</code>)</li>
	</ul>

	<h3><code>/api/maps</code></h3>
	</h4>Get maps. Parameters:</h4>
	<ul>
		<li><code>name</code>: <code>Option&lt;String&gt; (this can be an identifier just like above)</code></li>
		<li><code>tier</code>: <code>Option&lt;u8&gt;</code></li>
		<li><code>courses</code>: <code>Option&lt;u8&gt;</code></li>
		<li><code>validated</code>: <code>Option&lt;bool&gt;</code></li>
		<li><code>created_by</code>: <code>Option&lt;String&gt; (this is a player identifier just like above)</code></li>
		<li><code>approved_by</code>: <code>Option&lt;String&gt; (this is a player identifier just like above)</code></li>
		<li><code>limit</code>: <code>Option&lt;u32&gt;</code></li>
	</ul>

	<h3><code>/api/records/:id</code></h3>
	</h4>Get a record by its ID. This matches up with the GlobalAPI.</h4>

	</h4>Get records (sorted by date). Parameters:</h4>
	<ul>
		<li><code>mode</code>: <code>Option&lt;String&gt; (this can be an identifier just like above)</code></li>
		<li><code>stage</code>: <code>Option&lt;u8&gt;</code></li>
		<li><code>map</code>: <code>Option&lt;String&gt; (this can be an identifier just like above)</code></li>
		<li><code>player</code>: <code>Option&lt;String&gt; (this can be an identifier just like above)</code></li>
		<li><code>has_teleports</code>: <code>Option&lt;bool&gt;</code></li>
		<li><code>created_after</code>: <code>Option&lt;String&gt; (this is a date with the following format: <code>%Y-%m-%dT%H:%M:%S</code>)</code></li>
		<li><code>created_before</code>: <code>Option&lt;String&gt; (this is a date with the following format: <code>%Y-%m-%dT%H:%M:%S</code>)</code></li>
		<li><code>limit</code>: <code>Option&lt;u32&gt;</code></li>
	</ul>

	<h3><code>/api/records/top/player/:ident</code></h3>
	</h4>Get player personal bests (sorted by date). Parameters:</h4>
	<ul>
		<li><code>mode</code>: <code>Option&lt;String&gt; (this can be an identifier just like above)</code></li>
		<li><code>stage</code>: <code>Option&lt;u8&gt;</code></li>
		<li><code>map</code>: <code>Option&lt;String&gt; (this can be an identifier just like above)</code></li>
		<li><code>has_teleports</code>: <code>Option&lt;bool&gt;</code></li>
		<li><code>created_after</code>: <code>Option&lt;String&gt; (this is a date with the following format: <code>%Y-%m-%dT%H:%M:%S</code>)</code></li>
		<li><code>created_before</code>: <code>Option&lt;String&gt; (this is a date with the following format: <code>%Y-%m-%dT%H:%M:%S</code>)</code></li>
		<li><code>limit</code>: <code>Option&lt;u32&gt;</code></li>
	</ul>

	<h3><code>/api/records/top/map/:ident</code></h3>
	</h4>Get map leaderboards. Parameters:</h4>
	<ul>
		<li><code>mode</code>: <code>Option&lt;String&gt; (this can be an identifier just like above)</code></li>
		<li><code>stage</code>: <code>Option&lt;u8&gt;</code></li>
		<li><code>player</code>: <code>Option&lt;String&gt; (this can be an identifier just like above)</code></li>
		<li><code>has_teleports</code>: <code>Option&lt;bool&gt;</code></li>
		<li><code>created_after</code>: <code>Option&lt;String&gt; (this is a date with the following format: <code>%Y-%m-%dT%H:%M:%S</code>)</code></li>
		<li><code>created_before</code>: <code>Option&lt;String&gt; (this is a date with the following format: <code>%Y-%m-%dT%H:%M:%S</code>)</code></li>
		<li><code>limit</code>: <code>Option&lt;u32&gt;</code></li>
	</ul>
</body>
</html>
	"#)
}
