<template>
    <div>
        <div>Hello, World</div>
        <div><input v-on:click="plusOne()" type="button" :value="this.btnLabel"></div>
        <div>{{ counter }}</div>
    </div>
</template>
<script lang="ts">
import Vue from "vue";

export default Vue.extend({
    name: "index",

    head: function() {
        return {
            title: "Share_it"
        };
    },

    data: function() {
        return {
            btnLabel: "+1",
            counter: 1,
        };
    },

    methods: {
        plusOne: function() {
           this.counter++;
        },
    },
});
</script>