pub async fn index() -> axum::response::Html<&'static str> {
	r#"
<!DOCTYPE html>
<html lang="en">
<head>
	<meta charset="UTF-8">
	<meta http-equiv="X-UA-Compatible" content="IE=edge">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<title>(͡ ͡° ͜ つ ͡͡°)</title>
</head>
<style>

.schnose {
	display: flex;
	justify-content: center;
}

</style>
<body style="background-color: #1e1e2e;">
	<div class="schnose">
		<img src="https://media.discordapp.net/attachments/981130651094900756/1068608508645347408/schnose.png" alt="schnose" />
	</div>
</body>
</html>
	"#.into()
}
