import {
    Account,
    add_filter_category,
    CategoryType,
    delete_account_order,
    Filter,
    TransactionState,
    set_account_order_date,
    set_account_order_description,
    set_account_order_amount,
    set_account_order_resource,
    set_account_order_tags,
    set_account_order_state,
    get_account_categories,
    get_account_filtered_orders,
    load_account_data,
    remove_filter_category,
    serialize_account_as_yaml,
    sum_filtered_orders,
    toggle_account_order_visibility,
} from "money"

import {
    clearTableRows,
    getEnumStrings,
    enumStringToIndex,
    dateString,
} from "./modules/utils.js"

import {
    initCategoryFilter,
    resetCategoryFilter,
    initFilter,
    resourcesHideFilter,
    tagsHideFilter,
} from "./modules/order-filter.js"

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
const ordersTable = document.getElementById("orders")
// File management
const loadData = document.getElementById("load-data")
const downloadData = document.getElementById("download-data")
// Sum
const sum = document.getElementById("sum-canvas")
// Enumerations
const transactionStateEnum = getEnumStrings(TransactionState)
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
            // Checkboxes
            if (!filter_state) {
                resetCategoryFilter(type)
                initCategoryFilter(filter, type, list, render)
            }
        } else {
            console.error("Unable to get " + type + " categories")
        }
    }
}

const addOrderRow = (obj) => {
    var row = ordersTable.insertRow()
    const resourceList = get_account_categories(account, resourceCategoryType)
    const tagList = get_account_categories(account, tagCategoryType)

    // Date
    var date = document.createElement("input")
    date.type = "text"
    date.value = obj.order.date
    date.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (set_account_order_date(account, obj.id, date.value)) {
                console.log("Order " + obj.id + " date: " + date.value)
                requestAnimationFrame(render)
            } else {
                alert("Expected date format:  YY-MM-DD")
            }
        }
    })
    date.addEventListener('click', () => {
        date.value = dateString(new Date())
    })
    row.insertCell().appendChild(date)

    // Description node
    var description = document.createElement("input")
    description.type = "text"
    description.value = obj.order.description
    description.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (set_account_order_description(account, obj.id, description.value)) {
                console.log("Order " + obj.id + " description: " + description.value)
                requestAnimationFrame(render)
            }
        }
    })
    row.insertCell().appendChild(description)

    // Amount node
    var amount = document.createElement("input")
    amount.type = "text"
    amount.value = obj.order.amount.toFixed(2) + "€"
    amount.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (amount.value == "") {
                amount.value = "0.0"
            }

            const float = parseFloat(amount.value)
            if (set_account_order_amount(account, obj.id, float)) {
                console.log("Order " + obj.id + " amount: " + float.toFixed(2))
                requestAnimationFrame(render)
            }
        }
    })
    amount.addEventListener('click', event => {
        amount.value = ""
    })
    row.insertCell().appendChild(amount)

    // Resource node
    var resource = document.createElement("select")
    var empty_option = document.createElement("option")
    empty_option.value = "-"
    empty_option.text = "-"
    empty_option.disabled = true
    resource.appendChild(empty_option)
    resourceList.forEach(function(item) {
        var option = document.createElement("option")
        option.value = item
        option.text = item
        resource.appendChild(option)
    })
    if (obj.order.resource != null) {
        resource.value = obj.order.resource
    } else {
        resource.value = "-"
    }
    resource.addEventListener('change', function() {
        if (set_account_order_resource(account, obj.id, this.value)) {
            console.log("Order " + obj.id + " resource: " + this.value)
            requestAnimationFrame(render)
        }
    }, false)
    row.insertCell().appendChild(resource)

    // Tags
    var tags = document.createElement("select")
    tags.multiple = true
    tagList.forEach(function(item) {
        var option = document.createElement("option")
        option.value = item
        option.text = item

        obj.order.tags.forEach(function(tag) {
            if (item == tag) {
                option.selected = true
            }
        })

        tags.appendChild(option)
    })
    tags.addEventListener('change', function() {
        const selectedValues = [...this.options]
                     .filter((x) => x.selected)
                     .map((x)=>x.value)

        if (set_account_order_tags(account, obj.id, selectedValues)) {
            console.log("Order " + obj.id + " tags: " + selectedValues)
            requestAnimationFrame(render)
        } else {
            console.error("Unknown ", obj.id, selectedValues)
        }
    }, false)
    row.insertCell().appendChild(tags)

    // State
    var state = document.createElement("select")
    for (const entry of transactionStateEnum) {
        var option = document.createElement("option")
        option.text = entry[0]
        option.value = entry[0]
        state.appendChild(option)
    }
    state.value = obj.order.state
    state.addEventListener('change', function() {
        const index = enumStringToIndex(transactionStateEnum, this.value)
        if (set_account_order_state(account, obj.id, index)) {
            console.log("Order " + obj.id + " state: " + this.value)
            requestAnimationFrame(render)
        }
    }, false)
    row.insertCell().appendChild(state)

    // Remove button node
    if (obj.order.visible) {
        var remove_button = document.createElement("input")
        remove_button.type = "button"
        remove_button.value = "remove"
        remove_button.addEventListener('click', event => {
            if (toggle_account_order_visibility(account, obj.id)) {
                console.log("Order " + obj.id + ": removed!")
            }
            requestAnimationFrame(render)
        })
        row.insertCell().appendChild(remove_button)
    } else {
        var restore_button = document.createElement("input")
        var delete_button = document.createElement("input")
        restore_button.type = "button"
        restore_button.value = "restore"
        delete_button.type = "button"
        delete_button.value = "delete"
        restore_button.addEventListener('click', event => {
            if (toggle_account_order_visibility(account, obj.id)) {
                console.log("Order " + obj.id + ": restored!")
            }
            requestAnimationFrame(render)
        })
        delete_button.addEventListener('click', event => {
            if (confirm("Are you sure?")) {
                if (delete_account_order(account, obj.id)) {
                    console.log("Order " + obj.id + ": deleted!")
                }
                requestAnimationFrame(render)
            }
        })
        row.insertCell().appendChild(restore_button)
        row.insertCell().appendChild(delete_button)
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

    // Clear table rows
    clearTableRows(ordersTable)

    const orders = get_account_filtered_orders(account, filter)
    if (!orders.length == 0) {
        // Add table rows
        orders.forEach(function(item) {
            var obj = JSON.parse(item)
            addOrderRow(obj)
        })
    }

    // Sum
    sum.textContent = sum_filtered_orders(account, filter).toFixed(2) + '€'
}

initFilter(account, filter, render)
requestAnimationFrame(render)
