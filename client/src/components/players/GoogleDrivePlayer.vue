<template>
	<div class="googledrive">
		<video id="gdriveplayer" class="video-js vjs-default-skin" :key="videoId"></video>
	</div>
</template>

<script>
import videojs from "video.js";

export default {
	name: "GoogleDrivePlayer",
	props: {
		videoId: { type: String, required: true },
	},
	data() {
		return {
			player: null,
		};
	},
	computed: {
		videoSource() {
			// Yes, we send the google drive api key to the client. This is because we need to get the download link, but we can only do that
			// by authenticating with google, either by api key or by having people sign in with google. This is easier, and not really a problem
			// because we have 1,000,000,000 google drive api quota and the api methods we use don't cost that much. And this means we don't have
			// to waste bandwidth streaming video to clients.
			return `https://www.googleapis.com/drive/v3/files/${this.videoId}?key=${
				import.meta.env.GOOGLE_DRIVE_API_KEY
			}&alt=media&aknowledgeAbuse=true`;
		},
	},
	mounted() {
		this.player = videojs(document.getElementById("gdriveplayer"), {
			controls: false,
			responsive: true,
			loop: false,
			poster: this.$store.state.room.currentSource.thumbnail,
		});
		// required for iOS
		// this.player.setPlaysinline(true);
		this.player.on("ready", () => this.$emit("ready"));
		this.player.on("ended", () => this.$emit("end"));
		this.player.on("playing", () => this.$emit("playing"));
		this.player.on("pause", () => this.$emit("paused"));
		this.player.on("play", () => this.$emit("waiting"));
		this.player.on("stalled", () => this.$emit("buffering"));
		this.player.on("error", () => this.$emit("error"));
		this.loadVideoSource();
		this.player.load();
	},
	beforeUnmount() {
		if (this.player) {
			this.player.dispose();
		}
	},
	methods: {
		play() {
			return this.player.play();
		},
		pause() {
			return this.player.pause();
		},
		setVolume(volume) {
			return this.player.volume(volume / 100);
		},
		getPosition() {
			return this.player.currentTime();
		},
		setPosition(position) {
			return this.player.currentTime(position);
		},
		isCaptionsSupported() {
			return false;
		},
		getAvailablePlaybackRates() {
			return [1];
		},
		loadVideoSource() {
			this.player.src({
				src: this.videoSource,
				type: this.$store.state.room.currentSource.mime,
			});
		},
	},
	watch: {
		videoId() {
			this.loadVideoSource();
		},
	},
};
</script>

<style lang="scss" scoped>
@import url("https://vjs.zencdn.net/5.4.6/video-js.min.css");

.video-js {
	width: 100%;
	height: 100%;
}
</style>
