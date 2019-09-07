<template>
    <el-page-header @back="goBack" content="详情页面"></el-page-header>
</template>

<script>
    export default {
        name: "header",
        data:{

        },
        methods:{
            goBack() {
                console.log('go back');
            },
        }
    }
</script>

<style scoped>

</style>