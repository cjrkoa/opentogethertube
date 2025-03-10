<template>
	<div class="video-queue">
		<div class="empty-queue" v-if="store.state.room.queue.length === 0">
			<v-container style="height: 100%">
				<v-row justify="center" align="center" style="height: 100%">
					<div>
						<div class="msg">
							{{ $t("video-queue.no-videos") }}
						</div>
						<v-btn size="x-large" block @click="$emit('switchtab')">
							<v-icon style="margin-right: 8px">fa:fas fa-plus</v-icon>
							{{ $t("video-queue.add-video") }}
						</v-btn>
					</div>
				</v-row>
			</v-container>
		</div>
		<div class="queue-controls" v-if="store.state.room.queue.length > 0">
			<v-btn icon @click="roomapi.shuffle()">
				<v-icon>fa:fas fa-random</v-icon>
			</v-btn>
			<v-dialog v-model="exportDialog" width="600">
				<template v-slot:activator="{ props }">
					<v-btn v-bind="props">
						{{ $t("video-queue.export") }}
					</v-btn>
				</template>
				<v-card>
					<v-card-title>{{ $t("video-queue.export-diag-title") }}</v-card-title>
					<v-card-text>
						<span>{{ $t("video-queue.export-hint") }}</span>
						<v-textarea
							v-model="exportedQueue"
							readonly
							ref="exportTextBox"
							:class="copyExportSuccess ? 'text-success' : ''"
							:messages="copyExportSuccess ? $t('share-invite.copied') : ''"
						/>
					</v-card-text>
					<v-card-actions>
						<v-btn color="primary" @click="copyExported">
							{{ $t("common.copy") }}
						</v-btn>
						<v-btn @click="exportDialog = false">
							{{ $t("common.close") }}
						</v-btn>
					</v-card-actions>
				</v-card>
			</v-dialog>
		</div>
		<Sortable
			:list="store.state.room.queue"
			:move="() => granted('manage-queue.order')"
			@end="onQueueDragDrop"
			:options="{ animation: 200, handle: '.drag-handle' }"
			item-key="id"
		>
			<template #item="{ element, index }">
				<VideoQueueItem :key="element.id" :item="element" :index="index" />
			</template>
		</Sortable>
	</div>
</template>

<script lang="ts">
import { defineComponent, ref, computed } from "vue";
import VideoQueueItem from "@/components/VideoQueueItem.vue";
import { granted } from "@/util/grants";
import { useStore } from "@/store";
import { Sortable } from "sortablejs-vue3";
import { useConnection } from "@/plugins/connection";
import { useRoomApi } from "@/util/roomapi";
import { exportQueue } from "ott-common/queueexport";
import { useCopyFromTextbox } from "./composables";

const VideoQueue = defineComponent({
	name: "VideoQueue",
	components: {
		VideoQueueItem,
		Sortable,
	},
	setup() {
		const store = useStore();
		const roomapi = useRoomApi(useConnection());

		function onQueueDragDrop(e: { oldIndex: number; newIndex: number }) {
			roomapi.queueMove(e.oldIndex, e.newIndex);
		}

		const exportDialog = ref(false);
		const exportedQueue = computed(() => {
			const queue = [...store.state.room.queue];
			if (store.state.room.currentSource) {
				queue.unshift(store.state.room.currentSource);
			}
			return exportQueue(queue);
		});
		const exportTextBox = ref();
		const { copy: copyExported, copySuccess: copyExportSuccess } = useCopyFromTextbox(
			exportedQueue,
			exportTextBox
		);

		return {
			onQueueDragDrop,
			exportDialog,
			exportedQueue,
			exportTextBox,
			copyExported,
			copyExportSuccess,

			roomapi,
			granted,
			store,
		};
	},
});

export default VideoQueue;
</script>

<style lang="scss" scoped>
.video-queue {
	margin: 0 10px;
	min-height: 500px;
}

.empty-queue {
	height: 300px;

	.msg {
		opacity: 0.6;
		font-size: 20px;
	}
}

.queue-controls {
	margin-top: 6px;
}

// Transition animation
.video-queue-enter-active,
.video-queue-leave-active {
	transition: all 0.2s;
}
.video-queue-enter-from,
.video-queue-leave-to {
	opacity: 0;
	transform: translateX(-30px) scaleY(0);
}
.video-queue-move {
	transition: transform 0.2s;
}
</style>
