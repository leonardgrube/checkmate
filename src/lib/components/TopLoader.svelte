<div class="top-loader" role="status" aria-label="Loading">
	<div class="track">
		<div class="bar"></div>
	</div>
</div>

<style>
	.top-loader {
		width: 100%;
		padding-top: 2px;
	}

	.track {
		position: relative;
		width: 100%;
		height: 3px;
		border-radius: 999px;
		background: rgba(255, 255, 255, 0.06);
		overflow: hidden;
	}

	.bar {
		position: absolute;
		inset: 0 auto 0 0;
		width: 38%;
		border-radius: inherit;
		background: linear-gradient(
			90deg,
			rgba(74, 222, 128, 0.25),
			var(--accent),
			rgba(74, 222, 128, 0.25)
		);
		box-shadow: 0 0 10px var(--accent-dim);
		animation: slide 1.1s ease-in-out infinite;
	}

	@keyframes slide {
		0% {
			transform: translateX(-110%);
		}
		55% {
			transform: translateX(170%);
		}
		100% {
			transform: translateX(170%);
		}
	}
</style>
