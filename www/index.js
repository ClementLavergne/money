import { Account } from "money";

// Singleton
const account = new Account()
const inputTag = document.getElementById("tags");
const tags = document.getElementById("tag-list");
const addTag = document.getElementById("add-tag");
const removeTag = document.getElementById("remove-tag");

const refreshTags = (items) => {
    // Remove existing
    while (tags.firstChild) {
        tags.removeChild(tags.lastChild);
    }

    // Update
    items.forEach(function(item){
        var option = document.createElement('option');
        option.value = item;
        tags.appendChild(option);
    });

    // Clear text
    inputTag.value = "";
};

addTag.addEventListener("click", event => {
    account.add_tag(inputTag.value.toString())
    refreshTags(account.get_tags())
});

removeTag.addEventListener("click", event => {
    account.remove_tag(inputTag.value)
    refreshTags(account.get_tags())
});
