<template>
  <tr>
    <th scope="row">{{ index }}</th>
    <td>{{ entry.nickname }}</td>
    <td>{{ entry.time_joined }}</td>
    <td>
      <button @click="remove(user)" class="btn btn-outline-danger">
        <font-awesome-icon icon="minus-circle" />
        Remove
      </button>
    </td>
  </tr>
</template>

<script>
export default {
  props: {
    entry: {
      required: true,
      type: Object,
    },
    index: {
      required: true,
      type: Number,
    },
  },
};
</script>

<style scoped>
</style>