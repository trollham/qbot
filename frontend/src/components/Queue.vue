<template>
  <div class="queue">
    <div>
      <nav class="navbar navbar-dark">
        <strong>Queue size</strong>{{ queue.length }}
        <strong>Time remaining</strong>
        {{ (Math.floor(queue.length / pop_size) + 1) * 5 }} minutes
      </nav>
    </div>
    <div>
      <table class="table table-sm table-hover table-striped">
        <thead>
          <tr>
            <th scope="col">#</th>
            <th scope="col">Name</th>
            <th scope="col">Time</th>
            <th scope="col" v-if="is_owner">Actions</th>
          </tr>
        </thead>
        <tbody>
          <transition-group name="user-table">
            <QueueEntry
              v-for="(user, index) in queue"
              :key="user.id"
              :entry="user"
              :index="index + 1"
              :is_owner="is_owner"
              @remove-user="$emit('remove-user', user)"
              class="queue-item"
            ></QueueEntry>
          </transition-group>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script>
import QueueEntry from "./QueueEntry.vue";

export default {
  name: "Queue",
  components: { QueueEntry },
  data() {
    return {
      pop_size: 4,
    };
  },
  props: {
    is_open: {
      required: false,
      type: Boolean,
    },
    is_owner: {
      required: true,
      type: Boolean,
    },
    queue: {
      required: false,
      type: Array,
    },
  },
  emits: ["remove-user"],
};
</script>

<style scoped>
.queue {
  text-align: center;
}
.user-table-enter-active {
  animation: bounce-in 0.4s;
}
.user-table-leave-active {
  transition: opacity 300ms;
}
.user-table-leave-from {
  opacity: 1;
}
.user-table-leave-to {
  opacity: 0;
}
@keyframes bounce-in {
  0% {
    transform: scale(0);
  }
  50% {
    transform: scale(1.1);
  }
  100% {
    transform: scale(1);
  }
}
</style>
