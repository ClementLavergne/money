import {
    get_account_categories,
    get_filter_category_state,
    set_filter_categories,
    toggle_filter_category,
    clear_filter_categories,
    CategoryType,
    ItemSelector,
    TransactionState,
    VisibilityFilter,
    OrderingPreference,
    OrderingDirection,
} from "money"

import {
    enumIndexToString,
    enumStringToIndex,
    getEnumStrings,
    removeChildNodesByTagName,
    removeChildNodeById,
    firstDayCurrentMonthString,
    lastDayCurrentMonthString,
} from "./utils.js"

import {
    refreshCategoryTable,
} from "./category-table.js"

export {
    initFilter,
    initCategoryFilter,
    resetCategoryFilter,
    resourcesHideFilter,
    tagsHideFilter,
    dateHideFilter,
}

// Enumerations
const itemSelectorEnum = getEnumStrings(ItemSelector)
const categoryTypeEnum = getEnumStrings(CategoryType)
const transactionStateEnum = getEnumStrings(TransactionState)
const visibilityEnum = getEnumStrings(VisibilityFilter)
const orderingPreferenceEnum = getEnumStrings(OrderingPreference)
const orderingDirectionEnum = getEnumStrings(OrderingDirection)
// Category filters
const resourcesCluster = document.getElementById("resources-manager")
const resourcesFilterButton = document.getElementById("resources-filter-switch")
var resourcesHideFilter = true
const resourceCategoryType = enumStringToIndex(categoryTypeEnum, "Resource")
const tagsCluster = document.getElementById("tags-manager")
const tagsFilterButton = document.getElementById("tags-filter-switch")
var tagsHideFilter = true
const tagCategoryType = enumStringToIndex(categoryTypeEnum, "Tag")
// Date filter
const dateCluster = document.getElementById("date-manager")
const dateFilterButton = document.getElementById("date-filter-switch")
var dateHideFilter = true
// State filter
const statesCluster = document.getElementById("states-manager")
// Visibility filter
const visibilityManager = document.getElementById("visibility-manager")
// Ordering settings
const orderingPreferenceDiv = document.getElementById("ordering-preference")
const orderingDirectionDiv = document.getElementById("ordering-direction")

const initVisibilityFilter = (filter, render_func) => {
    visibilityEnum.forEach(function(item) {
        var label = document.createElement("label");
        label.innerHTML = item[0]

        var radioButton = document.createElement("input")
        radioButton.type = "radio"
        radioButton.id = item[0] + "Button"
        radioButton.name = "visibility"

        // Initialization
        if (filter.visibility == item[1]) {
            radioButton.checked = true
        }

        radioButton.addEventListener("click", event => {
            filter.visibility = item[1]
            radioButton.checked = true
            requestAnimationFrame(render_func)
        })

        var div = document.createElement("div")
        div.appendChild(radioButton)
        div.appendChild(label)
        visibilityManager.appendChild(div)
    })
}

const initOrderingFilter = (filter, render_func) => {
    orderingPreferenceEnum.forEach(function(item) {
        var label = document.createElement("label");
        label.innerHTML = item[0]

        var radioButton = document.createElement("input")
        radioButton.type = "radio"
        radioButton.id = item[0] + "Button"
        radioButton.name = "ordering"

        // Initialization
        if (filter.ordering == item[1]) {
            radioButton.checked = true
        }

        radioButton.addEventListener("click", event => {
            filter.ordering = item[1]
            radioButton.checked = true
            requestAnimationFrame(render_func)
        })

        var div = document.createElement("div")
        div.appendChild(radioButton)
        div.appendChild(label)
        orderingPreferenceDiv.appendChild(div)
    })

    orderingDirectionEnum.forEach(function(item) {
        var label = document.createElement("label");
        label.innerHTML = item[0]

        var radioButton = document.createElement("input")
        radioButton.type = "radio"
        radioButton.id = item[0] + "Button"
        radioButton.name = "direction"

        // Initialization
        if (filter.direction == item[1]) {
            radioButton.checked = true
        }

        radioButton.addEventListener("click", event => {
            filter.direction = item[1]
            radioButton.checked = true
            requestAnimationFrame(render_func)
        })

        var div = document.createElement("div")
        div.appendChild(radioButton)
        div.appendChild(label)
        orderingDirectionDiv.appendChild(div)
    })
}

const initStateFilter = (filter, render_func) => {
    transactionStateEnum.forEach(function(item) {
        var div = document.createElement("div")
        var label = document.createElement("label")
        var checkbox = document.createElement("input")
        label.appendChild(document.createTextNode(item[0]))
        checkbox.type = "checkbox"

        const index = enumStringToIndex(transactionStateEnum, item[0])

        // Initialize the checkbox with current value (should always be 'Selected')
        const string = enumIndexToString(itemSelectorEnum, filter.get_state(index))
        if (string == "Selected") {
            checkbox.checked = true
        } else if (string == "Discarded") {
            checkbox.checked = false
        } else {
            console.error("Unknown selector", string)
        }

        checkbox.addEventListener('change', function() {
            filter.toggle_state(index)
            requestAnimationFrame(render_func)
        }, false)

        div.appendChild(checkbox)
        div.appendChild(label)
        statesCluster.appendChild(div)
    })
}

const initDateRangeFilter = (account, filter, render_func) => {
    var div = document.createElement("div")
    div.class = "horizontal-container"
    var div_start = document.createElement("div")
    var div_stop = document.createElement("div")
    var label_start = document.createElement("label")
    var label_stop = document.createElement("label")
    label_start.appendChild(document.createTextNode("Start"))
    label_stop.appendChild(document.createTextNode("Stop"))
    var begin = document.createElement("input")
    begin.type = "text"
    begin.id = "date-filter-begin"
    var end = document.createElement("input")
    end.type = "text"
    end.id = "date-filter-end"

    begin.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (!filter.set_date_beginning(begin.value)) {
                begin.value = ""
            }
            refreshCategoryTable(account, "Resource", get_account_categories(account, resourceCategoryType), resourceCategoryType)
            refreshCategoryTable(account, "Tag", get_account_categories(account, tagCategoryType), tagCategoryType)
            requestAnimationFrame(render_func)
        }
    })

    begin.addEventListener('click', () => {
        if (begin.value == "") {
            begin.value = firstDayCurrentMonthString(new Date())
        }
    })

    end.addEventListener('keyup', ({key}) => {
        if (key === "Enter") {
            if (!filter.set_date_end(end.value)) {
                end.value = ""
            }
            refreshCategoryTable(account, "Resource", get_account_categories(account, resourceCategoryType), resourceCategoryType)
            refreshCategoryTable(account, "Tag", get_account_categories(account, tagCategoryType), tagCategoryType)
            requestAnimationFrame(render_func)
        }
    })

    end.addEventListener('click', () => {
        if (end.value == "") {
            end.value = lastDayCurrentMonthString(new Date())
        }
    })

    div_start.appendChild(begin)
    div_start.appendChild(label_start)
    div_stop.appendChild(end)
    div_stop.appendChild(label_stop)
    div.appendChild(div_start)
    div.appendChild(div_stop)
    dateCluster.appendChild(div)
}

const initFilter = (account, filter, render_func) => {
    // Enable/Disable resources filtering
    resourcesFilterButton.addEventListener("click", event => {
        const categoryType = resourceCategoryType

        // Toggle
        resourcesHideFilter = !resourcesHideFilter

        if (!resourcesHideFilter) {
            resourcesFilterButton.textContent = "disable filter"

            const resources = get_account_categories(account, categoryType)
            set_filter_categories(filter, categoryType, resources)
            initCategoryFilter(filter, "Resource", resources, render_func)
        } else {
            resourcesFilterButton.textContent = "enable filter"
            resetCategoryFilter("Resource")
            clear_filter_categories(filter, categoryType)
            requestAnimationFrame(render_func)
        }
    })

    // Enable/Disable tags filtering
    tagsFilterButton.addEventListener("click", event => {
        const categoryType = tagCategoryType

        // Toggle
        tagsHideFilter = !tagsHideFilter

        if (!tagsHideFilter) {
            tagsFilterButton.textContent = "disable filter"

            const tags = get_account_categories(account, categoryType)
            set_filter_categories(filter, categoryType, tags)
            initCategoryFilter(filter, "Tag", tags, render_func)
        } else {
            tagsFilterButton.textContent = "enable filter"
            resetCategoryFilter("Tag")
            clear_filter_categories(filter, categoryType)
            requestAnimationFrame(render_func)
        }
    })

    // Enable/Disable date filtering
    dateFilterButton.addEventListener("click", event => {
        // Toggle
        dateHideFilter = !dateHideFilter

        if (!dateHideFilter) {
            dateFilterButton.textContent = "disable filter"
            initDateRangeFilter(account, filter, render_func)
        } else {
            dateFilterButton.textContent = "enable filter"
            removeChildNodesByTagName(dateCluster, "DIV")
            filter.disable_date_option()
            refreshCategoryTable(account, "Resource", get_account_categories(account, resourceCategoryType), resourceCategoryType)
            refreshCategoryTable(account, "Tag", get_account_categories(account, tagCategoryType), tagCategoryType)
            requestAnimationFrame(render_func)
        }
    })

    // Initialize visibility filter
    initVisibilityFilter(filter, render_func)
    // Initialize state filter
    initStateFilter(filter, render_func)
    // Initialize ordering settings
    initOrderingFilter(filter, render_func)
}

const refreshCategoryFilter = (filter, categoryType, list) => {
    list.forEach(function(item) {
        const checkbox = document.getElementById("category-" + categoryType + "-" + item + "-checkbox")
        const string = enumIndexToString(itemSelectorEnum, get_filter_category_state(filter, categoryType, item))

        // Update checkbox state
        if (string == "Selected") {
            checkbox.checked = true
        } else if (string == "Discarded") {
            checkbox.checked = false
        } else {
            console.error("Unknown selector", string)
        }
    })
}

const initCategoryFilter = (filter, type, list, render_func) => {
    var node = undefined
    var categoryType = undefined
    const selectedIndex = enumStringToIndex(itemSelectorEnum, "Selected")
    const discardedIndex = enumStringToIndex(itemSelectorEnum, "Discarded")

    if (type == "Resource") {
        categoryType = enumStringToIndex(categoryTypeEnum, "Resource")
        node = resourcesCluster
    } else if (type == "Tag") {
        categoryType = enumStringToIndex(categoryTypeEnum, "Tag")
        node = tagsCluster
    } else {
        console.error("Unknown type: ", type)
    }

    if (node != null) {
        // Add button for deselecting/selecting all checkboxes
        if (list.length != 0) {
            var button = document.createElement("input")
            button.type = "button"
            button.value = "deselect all"
            button.id = "select-button"

            if (type == "Resource") {
                button.addEventListener('click', () => {
                    if (categoryType != undefined) {
                        if (button.value == "deselect all") {
                            button.value = "select all"

                            list.forEach(function(item) {
                                if (get_filter_category_state(filter, categoryType, item) == selectedIndex) {
                                    toggle_filter_category(filter, categoryType, item)
                                }
                            })
                        } else {
                            button.value = "deselect all"

                            list.forEach(function(item) {
                                if (get_filter_category_state(filter, categoryType, item) == discardedIndex) {
                                    toggle_filter_category(filter, categoryType, item)
                                }
                            })
                        }
                        refreshCategoryFilter(filter, categoryType, list)
                        requestAnimationFrame(render_func)
                    } else {
                        console.error("Invalid node: ", node)
                    }
                })
            } else {
                button.addEventListener('click', () => {
                    if (categoryType != undefined) {
                        list.forEach(function(item) {
                            if (get_filter_category_state(filter, categoryType, item) == selectedIndex) {
                                toggle_filter_category(filter, categoryType, item)
                            }
                        })
                        refreshCategoryFilter(filter, categoryType, list)
                        requestAnimationFrame(render_func)
                    } else {
                        console.error("Invalid node: ", node)
                    }
                })
            }

            node.appendChild(button)
        }

        // Add checkbox for each item
        list.forEach(function(item) {
            var div = document.createElement("div")
            var label = document.createElement("label")
            var checkbox = document.createElement("input")
            label.appendChild(document.createTextNode(item))
            checkbox.type = "checkbox"
            checkbox.id = "category-" + categoryType + "-" + item + "-checkbox"

            if (categoryType != undefined) {
                if (type == "Resource") {
                    // Select all resources
                    if (get_filter_category_state(filter, categoryType, item) != selectedIndex) {
                        toggle_filter_category(filter, categoryType, item)
                    }
                } else {
                    // Deselect all tags
                    if (get_filter_category_state(filter, categoryType, item) == selectedIndex) {
                        toggle_filter_category(filter, categoryType, item)
                    }
                }

                checkbox.addEventListener('change', function() {
                    if (toggle_filter_category(filter, categoryType, item) != undefined) {
                        requestAnimationFrame(render_func)
                    }
                    else {
                        console.error("Unknown selector", item)
                    }
                }, false)

                div.appendChild(checkbox)
                div.appendChild(label)
                node.appendChild(div)
            }
        })

        // Initialize the checkboxes
        if (categoryType != undefined) {
            refreshCategoryFilter(filter, categoryType, list)
        }
    }
}

const resetCategoryFilter = (type) => {
    var node = null
    if (type == "Resource") {
        node = resourcesCluster
    } else if (type == "Tag") {
        node = tagsCluster
    } else {
        console.error("Unknown type: ", type)
    }

    removeChildNodesByTagName(node, "DIV")
    removeChildNodeById(node, "select-button")
}
