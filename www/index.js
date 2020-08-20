import { Account } from "money";

// Singleton
const account = new Account()
// Tags management
const inputTag = document.getElementById("input-tag");
const tags = document.getElementById("tag-list");
const addTag = document.getElementById("add-tag");
const removeTag = document.getElementById("remove-tag");
// Resources management
const inputResource = document.getElementById("input-resource");
const resources = document.getElementById("resource-list");
const addResource = document.getElementById("add-resource");
const removeResource = document.getElementById("remove-resource");

const refreshList = (node, items) => {
    // Remove existing
    while (node.firstChild) {
        node.removeChild(node.lastChild);
    }

    // Update
    items.forEach(function(item){
        var option = document.createElement('option');
        option.value = item;
        node.appendChild(option);
    });
};

addTag.addEventListener("click", event => {
    account.add_tag(inputTag.value.toString())
    refreshList(tags, account.export_tags())
    inputTag.value = "";
});

removeTag.addEventListener("click", event => {
    account.remove_tag(inputTag.value)
    refreshList(tags, account.export_tags())
    inputTag.value = "";
});

addResource.addEventListener("click", event => {
    account.add_resource(inputResource.value.toString())
    refreshList(resources, account.export_resources())
    inputResource.value = "";
});

removeResource.addEventListener("click", event => {
    account.remove_resource(inputResource.value)
    refreshList(resources, account.export_resources())
    inputResource.value = "";
});
