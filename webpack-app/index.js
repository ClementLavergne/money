import {
    Account,
    add_filter_category,
    CategoryType,
    Filter,
    get_account_categories,
    load_account_data,
    remove_filter_category,
    serialize_account_as_yaml,
} from "money"

import {
    getEnumStrings,
    enumStringToIndex,
} from "./modules/utils.js"

import {
    initCategoryFilter,
    resetCategoryFilter,
    initFilter,
    resourcesHideFilter,
    tagsHideFilter,
} from "./modules/order-filter.js"

import {
    initOrderTable,
} from "./modules/order-table.js"

import {
    initCategoryTable,
    refreshCategoryTable,
} from "./modules/category-table.js"

// Singleton
const account = new Account()
const filter = new Filter()
// Tags management
const inputTag = document.getElementById("input-tag")
const tagsList = document.getElementById("tag-list")
const addTag = document.getElementById("add-tag")
const removeTag = document.getElementById("remove-tag")
// Resources management
const inputResource = document.getElementById("input-resource")
const resourcesList = document.getElementById("resource-list")
const addResource = document.getElementById("add-resource")
const removeResource = document.getElementById("remove-resource")
// Orders
const addOrder = document.getElementById("add-order")
// File management
const loadData = document.getElementById("load-data")
const downloadData = document.getElementById("download-data")
// Enumerations
const categoryTypeEnum = getEnumStrings(CategoryType)
const resourceCategoryType = enumStringToIndex(categoryTypeEnum, "Resource")
const tagCategoryType = enumStringToIndex(categoryTypeEnum, "Tag")

const refreshCategoryCombobox  = (node, list) => {
    // Remove all options
    while (node.firstChild) {
        node.removeChild(node.lastChild)
    }
    // Add options
    list.forEach(function(item) {
        var option = document.createElement('option')
        option.value = item
        node.appendChild(option)
    })
}

const refreshCategoryList = (type) => {
    var combobox
    var filter_state
    var categoryType = undefined
    var error = false
    switch (type) {
        case "Tag":
            combobox = tagsList
            categoryType = tagCategoryType
            filter_state = tagsHideFilter
            break;
        case "Resource":
            combobox = resourcesList
            categoryType = resourceCategoryType
            filter_state = resourcesHideFilter
            break;
        default:
            error = true
            console.error("Unknown category type: ", type)
    }

    if (error == false) {
        const list = get_account_categories(account, categoryType)
        if (list != undefined) {
            // Combobox
            refreshCategoryCombobox(combobox, list)
            // Results
            initCategoryTable(type, list)
            refreshCategoryTable(account, filter, type, list, categoryType)
            // Filter
            if (!filter_state) {
                resetCategoryFilter(type)
                initCategoryFilter(filter, type, list, render)
            }
        } else {
            console.error("Unable to get " + type + " categories")
        }
    }
}

addTag.addEventListener("click", event => {
    if (account.add_tag(inputTag.value) == undefined) {
        if (!tagsHideFilter) {
            add_filter_category(filter, tagCategoryType, inputTag.value)
        }
        refreshCategoryList("Tag")
        inputTag.value = ""
        requestAnimationFrame(render)
    }
})

removeTag.addEventListener("click", event => {
    if (account.remove_tag(inputTag.value) == undefined) {
        if (!tagsHideFilter) {
            remove_filter_category(filter, tagCategoryType, inputTag.value)
        }
        refreshCategoryList("Tag")
        inputTag.value = ""
        requestAnimationFrame(render)
    }
})

addResource.addEventListener("click", event => {
    if (account.add_resource(inputResource.value) == undefined) {
        if (!resourcesHideFilter) {
            add_filter_category(filter, resourceCategoryType, inputResource.value)
        }
        refreshCategoryList("Resource")
        inputResource.value = ""
        requestAnimationFrame(render)
    }
})

removeResource.addEventListener("click", event => {
    if (account.remove_resource(inputResource.value) == undefined) {
        if (!resourcesHideFilter) {
            remove_filter_category(filter, resourceCategoryType, inputResource.value)
        }
        refreshCategoryList("Resource")
        inputResource.value = ""
        requestAnimationFrame(render)
    }
})

addOrder.addEventListener("click", event => {
    account.add_order()
    requestAnimationFrame(render)
})

// Load YAML file
loadData.addEventListener("change", function() {
    var file = this.files[0]
    var reader = new FileReader()

    reader.readAsText(file,'UTF-8')

    reader.onload = readerEvent => {
        var content = readerEvent.target.result

        if (load_account_data(account, content)) {
            console.log("File '" + file + "' loaded!")
            refreshCategoryList("Resource")
            refreshCategoryList("Tag")
            requestAnimationFrame(render)
        }
    }
}, false)

// Write YAML file
downloadData.addEventListener("click", event => {
    function download(filename, text) {
        var element = document.createElement('a')
        element.setAttribute('href', 'data:text/plain;charset=utf-8,' + encodeURIComponent(text))
        element.setAttribute('download', filename)

        element.style.display = 'none'
        document.body.appendChild(element)

        element.click()

        document.body.removeChild(element)
    }

    const filename = prompt("Please enter file name:", "account-data.yml")

    if (filename != null) {
        const data = serialize_account_as_yaml(account)
        download(filename, data)
    }
})

const render = () => {
    console.log("Render!")

    // Display filtered orders
    initOrderTable(account, filter, render)
}

initFilter(account, filter, render)
requestAnimationFrame(render)
